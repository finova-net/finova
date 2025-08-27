package com.finova.reactnative;

import android.util.Base64;
import android.util.Log;
import com.facebook.react.bridge.*;
import com.facebook.react.modules.core.DeviceEventManagerModule;
import org.bouncycastle.crypto.generators.Argon2BytesGenerator;
import org.bouncycastle.crypto.params.Argon2Parameters;
import org.bouncycastle.jce.provider.BouncyCastleProvider;
import org.bouncycastle.util.encoders.Hex;
import org.tweetnacl.TweetNaclFast;

import javax.crypto.Cipher;
import javax.crypto.KeyGenerator;
import javax.crypto.Mac;
import javax.crypto.SecretKey;
import javax.crypto.spec.GCMParameterSpec;
import javax.crypto.spec.SecretKeySpec;
import java.nio.charset.StandardCharsets;
import java.security.*;
import java.security.spec.ECGenParameterSpec;
import java.security.spec.PKCS8EncodedKeySpec;
import java.security.spec.X509EncodedKeySpec;
import java.util.Arrays;
import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * Finova Network Crypto Utils Module for React Native Android
 * 
 * Enterprise-grade cryptographic utilities supporting:
 * - Solana wallet operations (Ed25519)
 * - Mining algorithm cryptography 
 * - Secure key management
 * - Anti-bot proof generation
 * - Referral code encryption
 * - NFT signature verification
 * - Cross-chain bridge cryptography
 * 
 * @version 1.0.0
 * @author Finova Network Team
 */
@ReactModule(name = CryptoUtilsModule.NAME)
public class CryptoUtilsModule extends ReactContextBaseJavaModule {
    public static final String NAME = "FinovaCryptoUtils";
    private static final String TAG = "FinovaCrypto";
    private static final int GCM_IV_LENGTH = 12;
    private static final int GCM_TAG_LENGTH = 16;
    private static final int ARGON2_ITERATIONS = 3;
    private static final int ARGON2_MEMORY = 4096;
    private static final int ARGON2_PARALLELISM = 1;
    
    private final ExecutorService cryptoExecutor;
    private final SecureRandom secureRandom;
    
    static {
        Security.addProvider(new BouncyCastleProvider());
    }

    public CryptoUtilsModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.cryptoExecutor = Executors.newFixedThreadPool(4);
        this.secureRandom = new SecureRandom();
    }

    @Override
    public String getName() {
        return NAME;
    }

    @Override
    public Map<String, Object> getConstants() {
        Map<String, Object> constants = new HashMap<>();
        constants.put("SOLANA_KEYPAIR_LENGTH", 64);
        constants.put("ED25519_SIGNATURE_LENGTH", 64);
        constants.put("MINING_NONCE_LENGTH", 32);
        constants.put("REFERRAL_CODE_LENGTH", 12);
        constants.put("PROOF_OF_WORK_DIFFICULTY", 4);
        return constants;
    }

    /**
     * Generate Solana Ed25519 keypair for wallet operations
     */
    @ReactMethod
    public void generateSolanaKeypair(Promise promise) {
        cryptoExecutor.execute(() -> {
            try {
                // Generate Ed25519 keypair using TweetNaCl
                TweetNaclFast.Signature.KeyPair keyPair = TweetNaclFast.Signature.keyPair();
                
                WritableMap result = Arguments.createMap();
                result.putString("publicKey", Base64.encodeToString(keyPair.getPublicKey(), Base64.NO_WRAP));
                result.putString("privateKey", Base64.encodeToString(keyPair.getSecretKey(), Base64.NO_WRAP));
                result.putString("address", deriveSolanaAddress(keyPair.getPublicKey()));
                
                promise.resolve(result);
            } catch (Exception e) {
                Log.e(TAG, "Failed to generate Solana keypair", e);
                promise.reject("KEYPAIR_GENERATION_ERROR", e.getMessage(), e);
            }
        });
    }

    /**
     * Sign transaction with Ed25519 for Solana blockchain
     */
    @ReactMethod
    public void signTransaction(String privateKeyBase64, String transactionData, Promise promise) {
        cryptoExecutor.execute(() -> {
            try {
                byte[] privateKey = Base64.decode(privateKeyBase64, Base64.NO_WRAP);
                byte[] message = transactionData.getBytes(StandardCharsets.UTF_8);
                
                // Create signature using TweetNaCl
                TweetNaclFast.Signature signer = new TweetNaclFast.Signature(null, privateKey);
                byte[] signature = signer.detached(message);
                
                WritableMap result = Arguments.createMap();
                result.putString("signature", Base64.encodeToString(signature, Base64.NO_WRAP));
                result.putString("publicKey", Base64.encodeToString(
                    Arrays.copyOfRange(privateKey, 32, 64), Base64.NO_WRAP));
                
                promise.resolve(result);
            } catch (Exception e) {
                Log.e(TAG, "Failed to sign transaction", e);
                promise.reject("SIGNING_ERROR", e.getMessage(), e);
            }
        });
    }

    /**
     * Verify Ed25519 signature for transaction validation
     */
    @ReactMethod
    public void verifySignature(String publicKeyBase64, String signatureBase64, 
                               String message, Promise promise) {
        cryptoExecutor.execute(() -> {
            try {
                byte[] publicKey = Base64.decode(publicKeyBase64, Base64.NO_WRAP);
                byte[] signature = Base64.decode(signatureBase64, Base64.NO_WRAP);
                byte[] messageBytes = message.getBytes(StandardCharsets.UTF_8);
                
                TweetNaclFast.Signature verifier = new TweetNaclFast.Signature(publicKey, null);
                boolean isValid = verifier.detached_verify(messageBytes, signature);
                
                promise.resolve(isValid);
            } catch (Exception e) {
                Log.e(TAG, "Failed to verify signature", e);
                promise.reject("VERIFICATION_ERROR", e.getMessage(), e);
            }
        });
    }

    /**
     * Generate mining proof-of-work for Finova mining algorithm
     */
    @ReactMethod
    public void generateMiningProof(String userAddress, int difficulty, 
                                   double currentRate, Promise promise) {
        cryptoExecutor.execute(() -> {
            try {
                long timestamp = System.currentTimeMillis();
                byte[] nonce = new byte[32];
                secureRandom.nextBytes(nonce);
                
                // Create mining challenge
                String challenge = userAddress + timestamp + currentRate;
                byte[] challengeBytes = challenge.getBytes(StandardCharsets.UTF_8);
                
                // Find valid nonce
                MessageDigest sha256 = MessageDigest.getInstance("SHA-256");
                byte[] hash;
                int attempts = 0;
                
                do {
                    // Increment nonce
                    incrementNonce(nonce);
                    attempts++;
                    
                    // Calculate hash
                    sha256.reset();
                    sha256.update(challengeBytes);
                    sha256.update(nonce);
                    hash = sha256.digest();
                    
                } while (!isValidProof(hash, difficulty) && attempts < 1000000);
                
                if (attempts >= 1000000) {
                    promise.reject("MINING_TIMEOUT", "Failed to find valid proof within timeout");
                    return;
                }
                
                // Calculate mining score based on proof quality
                double miningScore = calculateMiningScore(hash, attempts, currentRate);
                
                WritableMap result = Arguments.createMap();
                result.putString("nonce", Base64.encodeToString(nonce, Base64.NO_WRAP));
                result.putString("hash", Hex.toHexString(hash));
                result.putInt("attempts", attempts);
                result.putDouble("miningScore", miningScore);
                result.putDouble("timestamp", timestamp);
                result.putString("challenge", challenge);
                
                promise.resolve(result);
            } catch (Exception e) {
                Log.e(TAG, "Failed to generate mining proof", e);
                promise.reject("MINING_PROOF_ERROR", e.getMessage(), e);
            }
        });
    }

    /**
     * Generate secure referral code with cryptographic verification
     */
    @ReactMethod
    public void generateReferralCode(String userAddress, String secretKey, Promise promise) {
        cryptoExecutor.execute(() -> {
            try {
                // Create unique identifier from user address
                MessageDigest sha256 = MessageDigest.getInstance("SHA-256");
                byte[] addressHash = sha256.digest(userAddress.getBytes(StandardCharsets.UTF_8));
                
                // Generate referral code using first 8 bytes + checksum
                byte[] codeBytes = Arrays.copyOf(addressHash, 8);
                byte[] checksum = calculateChecksum(codeBytes, secretKey);
                
                // Combine code + checksum for verification
                byte[] fullCode = new byte[12];
                System.arraycopy(codeBytes, 0, fullCode, 0, 8);
                System.arraycopy(checksum, 0, fullCode, 8, 4);
                
                // Encode as human-readable string
                String referralCode = encodeReferralCode(fullCode);
                
                WritableMap result = Arguments.createMap();
                result.putString("referralCode", referralCode);
                result.putString("userAddress", userAddress);
                result.putDouble("timestamp", System.currentTimeMillis());
                
                promise.resolve(result);
            } catch (Exception e) {
                Log.e(TAG, "Failed to generate referral code", e);
                promise.reject("REFERRAL_CODE_ERROR", e.getMessage(), e);
            }
        });
    }

    /**
     * Verify referral code authenticity and extract user address
     */
    @ReactMethod
    public void verifyReferralCode(String referralCode, String secretKey, Promise promise) {
        cryptoExecutor.execute(() -> {
            try {
                // Decode referral code
                byte[] codeBytes = decodeReferralCode(referralCode);
                if (codeBytes.length != 12) {
                    promise.reject("INVALID_CODE", "Invalid referral code format");
                    return;
                }
                
                // Extract components
                byte[] addressPart = Arrays.copyOfRange(codeBytes, 0, 8);
                byte[] providedChecksum = Arrays.copyOfRange(codeBytes, 8, 12);
                
                // Verify checksum
                byte[] expectedChecksum = calculateChecksum(addressPart, secretKey);
                boolean isValid = Arrays.equals(providedChecksum, expectedChecksum);
                
                WritableMap result = Arguments.createMap();
                result.putBoolean("isValid", isValid);
                if (isValid) {
                    result.putString("addressHash", Hex.toHexString(addressPart));
                }
                
                promise.resolve(result);
            } catch (Exception e) {
                Log.e(TAG, "Failed to verify referral code", e);
                promise.reject("REFERRAL_VERIFY_ERROR", e.getMessage(), e);
            }
        });
    }

    /**
     * Encrypt sensitive user data using AES-GCM
     */
    @ReactMethod
    public void encryptUserData(String data, String password, Promise promise) {
        cryptoExecutor.execute(() -> {
            try {
                // Derive key using Argon2
                byte[] salt = new byte[32];
                secureRandom.nextBytes(salt);
                byte[] key = deriveKeyArgon2(password, salt);
                
                // Generate IV
                byte[] iv = new byte[GCM_IV_LENGTH];
                secureRandom.nextBytes(iv);
                
                // Encrypt data
                Cipher cipher = Cipher.getInstance("AES/GCM/NoPadding");
                SecretKeySpec keySpec = new SecretKeySpec(key, "AES");
                GCMParameterSpec gcmSpec = new GCMParameterSpec(GCM_TAG_LENGTH * 8, iv);
                cipher.init(Cipher.ENCRYPT_MODE, keySpec, gcmSpec);
                
                byte[] ciphertext = cipher.doFinal(data.getBytes(StandardCharsets.UTF_8));
                
                WritableMap result = Arguments.createMap();
                result.putString("encryptedData", Base64.encodeToString(ciphertext, Base64.NO_WRAP));
                result.putString("salt", Base64.encodeToString(salt, Base64.NO_WRAP));
                result.putString("iv", Base64.encodeToString(iv, Base64.NO_WRAP));
                
                promise.resolve(result);
            } catch (Exception e) {
                Log.e(TAG, "Failed to encrypt data", e);
                promise.reject("ENCRYPTION_ERROR", e.getMessage(), e);
            }
        });
    }

    /**
     * Decrypt user data using AES-GCM
     */
    @ReactMethod
    public void decryptUserData(String encryptedDataBase64, String saltBase64, 
                               String ivBase64, String password, Promise promise) {
        cryptoExecutor.execute(() -> {
            try {
                byte[] encryptedData = Base64.decode(encryptedDataBase64, Base64.NO_WRAP);
                byte[] salt = Base64.decode(saltBase64, Base64.NO_WRAP);
                byte[] iv = Base64.decode(ivBase64, Base64.NO_WRAP);
                
                // Derive key
                byte[] key = deriveKeyArgon2(password, salt);
                
                // Decrypt data
                Cipher cipher = Cipher.getInstance("AES/GCM/NoPadding");
                SecretKeySpec keySpec = new SecretKeySpec(key, "AES");
                GCMParameterSpec gcmSpec = new GCMParameterSpec(GCM_TAG_LENGTH * 8, iv);
                cipher.init(Cipher.DECRYPT_MODE, keySpec, gcmSpec);
                
                byte[] plaintext = cipher.doFinal(encryptedData);
                String data = new String(plaintext, StandardCharsets.UTF_8);
                
                promise.resolve(data);
            } catch (Exception e) {
                Log.e(TAG, "Failed to decrypt data", e);
                promise.reject("DECRYPTION_ERROR", e.getMessage(), e);
            }
        });
    }

    /**
     * Generate anti-bot proof based on user behavior patterns
     */
    @ReactMethod
    public void generateAntiBotProof(ReadableMap behaviorData, Promise promise) {
        cryptoExecutor.execute(() -> {
            try {
                // Extract behavior parameters
                long sessionDuration = (long) behaviorData.getDouble("sessionDuration");
                int clickCount = behaviorData.getInt("clickCount");
                double avgClickInterval = behaviorData.getDouble("avgClickInterval");
                String deviceFingerprint = behaviorData.getString("deviceFingerprint");
                
                // Calculate human probability score
                double humanScore = calculateHumanProbability(
                    sessionDuration, clickCount, avgClickInterval, deviceFingerprint);
                
                // Generate cryptographic proof
                MessageDigest sha256 = MessageDigest.getInstance("SHA-256");
                String proofData = String.format("%.6f|%d|%.3f|%s|%d", 
                    humanScore, sessionDuration, avgClickInterval, 
                    deviceFingerprint, System.currentTimeMillis());
                
                byte[] proofHash = sha256.digest(proofData.getBytes(StandardCharsets.UTF_8));
                
                WritableMap result = Arguments.createMap();
                result.putDouble("humanScore", humanScore);
                result.putString("proofHash", Hex.toHexString(proofHash));
                result.putString("proofData", proofData);
                result.putBoolean("isHuman", humanScore > 0.7);
                
                promise.resolve(result);
            } catch (Exception e) {
                Log.e(TAG, "Failed to generate anti-bot proof", e);
                promise.reject("ANTIBOT_PROOF_ERROR", e.getMessage(), e);
            }
        });
    }

    /**
     * Generate NFT signature for marketplace transactions
     */
    @ReactMethod
    public void signNFTTransaction(String privateKey, ReadableMap nftData, Promise promise) {
        cryptoExecutor.execute(() -> {
            try {
                // Extract NFT transaction data
                String tokenId = nftData.getString("tokenId");
                String owner = nftData.getString("owner");
                String buyer = nftData.getString("buyer");
                double price = nftData.getDouble("price");
                long timestamp = (long) nftData.getDouble("timestamp");
                
                // Create transaction message
                String message = String.format("%s|%s|%s|%.6f|%d", 
                    tokenId, owner, buyer, price, timestamp);
                
                // Sign with Ed25519
                byte[] privateKeyBytes = Base64.decode(privateKey, Base64.NO_WRAP);
                TweetNaclFast.Signature signer = new TweetNaclFast.Signature(null, privateKeyBytes);
                byte[] signature = signer.detached(message.getBytes(StandardCharsets.UTF_8));
                
                WritableMap result = Arguments.createMap();
                result.putString("signature", Base64.encodeToString(signature, Base64.NO_WRAP));
                result.putString("message", message);
                result.putString("publicKey", Base64.encodeToString(
                    Arrays.copyOfRange(privateKeyBytes, 32, 64), Base64.NO_WRAP));
                
                promise.resolve(result);
            } catch (Exception e) {
                Log.e(TAG, "Failed to sign NFT transaction", e);
                promise.reject("NFT_SIGNING_ERROR", e.getMessage(), e);
            }
        });
    }

    /**
     * Hash user activity for XP calculation integrity
     */
    @ReactMethod
    public void hashUserActivity(ReadableMap activityData, Promise promise) {
        cryptoExecutor.execute(() -> {
            try {
                String platform = activityData.getString("platform");
                String activityType = activityData.getString("type");
                String content = activityData.getString("content");
                long timestamp = (long) activityData.getDouble("timestamp");
                String userId = activityData.getString("userId");
                
                // Create activity hash for integrity verification
                MessageDigest sha256 = MessageDigest.getInstance("SHA-256");
                String activityString = String.format("%s|%s|%s|%d|%s", 
                    platform, activityType, content, timestamp, userId);
                
                byte[] hash = sha256.digest(activityString.getBytes(StandardCharsets.UTF_8));
                
                // Create HMAC for additional security
                Mac hmac = Mac.getInstance("HmacSHA256");
                SecretKeySpec hmacKey = new SecretKeySpec(
                    userId.getBytes(StandardCharsets.UTF_8), "HmacSHA256");
                hmac.init(hmacKey);
                byte[] hmacResult = hmac.doFinal(hash);
                
                WritableMap result = Arguments.createMap();
                result.putString("activityHash", Hex.toHexString(hash));
                result.putString("hmacSignature", Hex.toHexString(hmacResult));
                result.putString("activityData", activityString);
                
                promise.resolve(result);
            } catch (Exception e) {
                Log.e(TAG, "Failed to hash user activity", e);
                promise.reject("ACTIVITY_HASH_ERROR", e.getMessage(), e);
            }
        });
    }

    // Private utility methods

    private String deriveSolanaAddress(byte[] publicKey) {
        return Base64.encodeToString(publicKey, Base64.NO_WRAP);
    }

    private void incrementNonce(byte[] nonce) {
        for (int i = nonce.length - 1; i >= 0; i--) {
            if (++nonce[i] != 0) break;
        }
    }

    private boolean isValidProof(byte[] hash, int difficulty) {
        int zeroBits = 0;
        for (byte b : hash) {
            if (b == 0) {
                zeroBits += 8;
            } else {
                zeroBits += Integer.numberOfLeadingZeros(b & 0xFF) - 24;
                break;
            }
        }
        return zeroBits >= difficulty;
    }

    private double calculateMiningScore(byte[] hash, int attempts, double currentRate) {
        // Calculate quality score based on proof efficiency
        double efficiency = Math.min(1.0, 1000.0 / attempts);
        double hashQuality = calculateHashQuality(hash);
        return currentRate * efficiency * hashQuality;
    }

    private double calculateHashQuality(byte[] hash) {
        // Analyze hash entropy and distribution
        int[] distribution = new int[256];
        for (byte b : hash) {
            distribution[b & 0xFF]++;
        }
        
        double entropy = 0.0;
        for (int count : distribution) {
            if (count > 0) {
                double probability = (double) count / hash.length;
                entropy -= probability * Math.log(probability) / Math.log(2);
            }
        }
        
        return Math.min(1.0, entropy / 8.0);
    }

    private byte[] calculateChecksum(byte[] data, String secret) throws Exception {
        Mac hmac = Mac.getInstance("HmacSHA256");
        SecretKeySpec key = new SecretKeySpec(secret.getBytes(StandardCharsets.UTF_8), "HmacSHA256");
        hmac.init(key);
        byte[] fullHash = hmac.doFinal(data);
        return Arrays.copyOf(fullHash, 4);
    }

    private String encodeReferralCode(byte[] codeBytes) {
        // Custom base32-like encoding for readability
        String alphabet = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        StringBuilder result = new StringBuilder();
        
        for (int i = 0; i < codeBytes.length; i += 5) {
            long value = 0;
            int byteCount = Math.min(5, codeBytes.length - i);
            
            for (int j = 0; j < byteCount; j++) {
                value = (value << 8) | (codeBytes[i + j] & 0xFF);
            }
            
            for (int j = 0; j < (byteCount * 8 + 4) / 5; j++) {
                result.append(alphabet.charAt((int) (value & 0x1F)));
                value >>= 5;
            }
        }
        
        return result.reverse().toString();
    }

    private byte[] decodeReferralCode(String code) {
        // Decode custom base32-like encoding
        String alphabet = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        StringBuilder binary = new StringBuilder();
        
        for (char c : code.toCharArray()) {
            int value = alphabet.indexOf(Character.toUpperCase(c));
            if (value == -1) continue;
            
            String bits = String.format("%5s", Integer.toBinaryString(value)).replace(' ', '0');
            binary.append(bits);
        }
        
        byte[] result = new byte[binary.length() / 8];
        for (int i = 0; i < result.length; i++) {
            String byteBits = binary.substring(i * 8, (i + 1) * 8);
            result[i] = (byte) Integer.parseInt(byteBits, 2);
        }
        
        return result;
    }

    private byte[] deriveKeyArgon2(String password, byte[] salt) {
        Argon2Parameters params = new Argon2Parameters.Builder(Argon2Parameters.ARGON2_id)
                .withVersion(Argon2Parameters.ARGON2_VERSION_13)
                .withIterations(ARGON2_ITERATIONS)
                .withMemoryAsKB(ARGON2_MEMORY)
                .withParallelism(ARGON2_PARALLELISM)
                .withSalt(salt)
                .build();
        
        Argon2BytesGenerator generator = new Argon2BytesGenerator();
        generator.init(params);
        
        byte[] key = new byte[32];
        generator.generateBytes(password.getBytes(StandardCharsets.UTF_8), key);
        return key;
    }

    private double calculateHumanProbability(long sessionDuration, int clickCount, 
                                           double avgClickInterval, String deviceFingerprint) {
        double score = 0.5; // Base score
        
        // Session duration analysis
        if (sessionDuration > 30000 && sessionDuration < 3600000) { // 30s - 1h
            score += 0.2;
        }
        
        // Click pattern analysis
        if (avgClickInterval > 500 && avgClickInterval < 5000) { // 0.5s - 5s
            score += 0.2;
        }
        
        // Click count reasonableness
        double clickRate = (double) clickCount / (sessionDuration / 1000.0);
        if (clickRate > 0.1 && clickRate < 2.0) { // 0.1 - 2.0 clicks per second
            score += 0.1;
        }
        
        // Device fingerprint consistency
        if (deviceFingerprint != null && deviceFingerprint.length() > 10) {
            score += 0.1;
        }
        
        // Random variance (human imperfection)
        double variance = Math.abs(avgClickInterval - 1000) / 1000.0;
        if (variance > 0.1 && variance < 0.5) {
            score += 0.1;
        }
        
        return Math.min(1.0, Math.max(0.0, score));
    }

    @Override
    public void onCatalystInstanceDestroy() {
        super.onCatalystInstanceDestroy();
        if (cryptoExecutor != null && !cryptoExecutor.isShutdown()) {
            cryptoExecutor.shutdown();
        }
    }
}
