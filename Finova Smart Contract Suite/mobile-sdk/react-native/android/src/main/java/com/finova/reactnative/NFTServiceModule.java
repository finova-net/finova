package com.finova.reactnative;

import android.util.Log;
import androidx.annotation.NonNull;
import androidx.annotation.Nullable;

import com.facebook.react.bridge.Arguments;
import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import com.facebook.react.bridge.ReadableMap;
import com.facebook.react.bridge.WritableArray;
import com.facebook.react.bridge.WritableMap;
import com.facebook.react.modules.core.DeviceEventManagerModule;

import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

import java.io.IOException;
import java.math.BigDecimal;
import java.util.HashMap;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import okhttp3.Call;
import okhttp3.Callback;
import okhttp3.MediaType;
import okhttp3.OkHttpClient;
import okhttp3.Request;
import okhttp3.RequestBody;
import okhttp3.Response;

/**
 * Finova Network NFT Service Module for React Native Android
 * Handles Special Cards, Profile Badges, and Achievement NFTs
 * 
 * Features:
 * - Special Card Management (Mining Boost, XP Accelerator, Referral Power)
 * - Profile Badge NFTs with utility bonuses
 * - Achievement NFT minting and tracking
 * - NFT Marketplace integration
 * - Card synergy calculations
 * 
 * @author Finova Network Team
 * @version 1.0.0
 */
public class NFTServiceModule extends ReactContextBaseJavaModule {
    
    private static final String TAG = "FinovaNFTService";
    private static final String MODULE_NAME = "FinovaNFTService";
    
    // API Endpoints
    private static final String BASE_API_URL = "https://api.finova.network/v1";
    private static final String NFT_ENDPOINT = BASE_API_URL + "/nft";
    private static final String CARDS_ENDPOINT = BASE_API_URL + "/cards";
    private static final String MARKETPLACE_ENDPOINT = BASE_API_URL + "/marketplace";
    
    // Special Card Categories
    public enum CardCategory {
        MINING_BOOST("mining_boost"),
        XP_ACCELERATOR("xp_accelerator"),
        REFERRAL_POWER("referral_power"),
        PROFILE_BADGE("profile_badge"),
        ACHIEVEMENT("achievement");
        
        private final String value;
        CardCategory(String value) { this.value = value; }
        public String getValue() { return value; }
    }
    
    // Card Rarity Levels
    public enum CardRarity {
        COMMON("common", 1.0, 0.0),
        UNCOMMON("uncommon", 1.0, 0.05),
        RARE("rare", 1.0, 0.10),
        EPIC("epic", 1.0, 0.20),
        LEGENDARY("legendary", 1.0, 0.35);
        
        private final String name;
        private final double baseMultiplier;
        private final double rarityBonus;
        
        CardRarity(String name, double baseMultiplier, double rarityBonus) {
            this.name = name;
            this.baseMultiplier = baseMultiplier;
            this.rarityBonus = rarityBonus;
        }
        
        public String getName() { return name; }
        public double getBaseMultiplier() { return baseMultiplier; }
        public double getRarityBonus() { return rarityBonus; }
    }
    
    private final ReactApplicationContext reactContext;
    private final OkHttpClient httpClient;
    private final ExecutorService executorService;
    private String userToken;
    private String walletAddress;
    
    public NFTServiceModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.reactContext = reactContext;
        this.httpClient = new OkHttpClient.Builder()
                .connectTimeout(30, java.util.concurrent.TimeUnit.SECONDS)
                .readTimeout(60, java.util.concurrent.TimeUnit.SECONDS)
                .build();
        this.executorService = Executors.newFixedThreadPool(4);
    }
    
    @Override
    @NonNull
    public String getName() {
        return MODULE_NAME;
    }
    
    @Override
    public Map<String, Object> getConstants() {
        final Map<String, Object> constants = new HashMap<>();
        
        // Card Categories
        Map<String, String> categories = new HashMap<>();
        for (CardCategory category : CardCategory.values()) {
            categories.put(category.name(), category.getValue());
        }
        constants.put("CARD_CATEGORIES", categories);
        
        // Card Rarities
        Map<String, Object> rarities = new HashMap<>();
        for (CardRarity rarity : CardRarity.values()) {
            Map<String, Object> rarityData = new HashMap<>();
            rarityData.put("name", rarity.getName());
            rarityData.put("baseMultiplier", rarity.getBaseMultiplier());
            rarityData.put("rarityBonus", rarity.getRarityBonus());
            rarities.put(rarity.name(), rarityData);
        }
        constants.put("CARD_RARITIES", rarities);
        
        // Special Card Effects
        Map<String, Object> effects = new HashMap<>();
        effects.put("DOUBLE_MINING", createCardEffect("Double Mining", 2.0, 24, 50));
        effects.put("TRIPLE_MINING", createCardEffect("Triple Mining", 3.0, 12, 150));
        effects.put("MINING_FRENZY", createCardEffect("Mining Frenzy", 6.0, 4, 500));
        effects.put("XP_DOUBLE", createCardEffect("XP Double", 2.0, 24, 40));
        effects.put("REFERRAL_BOOST", createCardEffect("Referral Boost", 1.5, 168, 60));
        constants.put("CARD_EFFECTS", effects);
        
        return constants;
    }
    
    private Map<String, Object> createCardEffect(String name, double multiplier, int durationHours, int price) {
        Map<String, Object> effect = new HashMap<>();
        effect.put("name", name);
        effect.put("multiplier", multiplier);
        effect.put("durationHours", durationHours);
        effect.put("priceFIN", price);
        return effect;
    }
    
    /**
     * Initialize NFT Service with user credentials
     */
    @ReactMethod
    public void initialize(String token, String wallet, Promise promise) {
        try {
            this.userToken = token;
            this.walletAddress = wallet;
            
            Log.d(TAG, "NFT Service initialized for wallet: " + wallet);
            
            WritableMap result = Arguments.createMap();
            result.putString("status", "success");
            result.putString("message", "NFT Service initialized successfully");
            result.putString("walletAddress", wallet);
            
            promise.resolve(result);
            
        } catch (Exception e) {
            Log.e(TAG, "Failed to initialize NFT Service", e);
            promise.reject("INIT_ERROR", "Failed to initialize NFT Service: " + e.getMessage());
        }
    }
    
    /**
     * Get user's NFT collection including Special Cards and Badges
     */
    @ReactMethod
    public void getUserNFTCollection(Promise promise) {
        if (!isInitialized()) {
            promise.reject("NOT_INITIALIZED", "NFT Service not initialized");
            return;
        }
        
        executorService.execute(() -> {
            try {
                Request request = new Request.Builder()
                        .url(NFT_ENDPOINT + "/collection/" + walletAddress)
                        .addHeader("Authorization", "Bearer " + userToken)
                        .addHeader("Content-Type", "application/json")
                        .get()
                        .build();
                
                httpClient.newCall(request).enqueue(new Callback() {
                    @Override
                    public void onFailure(@NonNull Call call, @NonNull IOException e) {
                        Log.e(TAG, "Failed to fetch NFT collection", e);
                        promise.reject("FETCH_ERROR", "Failed to fetch NFT collection: " + e.getMessage());
                    }
                    
                    @Override
                    public void onResponse(@NonNull Call call, @NonNull Response response) throws IOException {
                        try {
                            String responseBody = response.body().string();
                            JSONObject jsonResponse = new JSONObject(responseBody);
                            
                            if (response.isSuccessful()) {
                                WritableMap collection = parseNFTCollection(jsonResponse);
                                promise.resolve(collection);
                                
                                // Emit event for real-time updates
                                emitEvent("nftCollectionUpdated", collection);
                                
                            } else {
                                promise.reject("API_ERROR", "API Error: " + jsonResponse.optString("message", "Unknown error"));
                            }
                            
                        } catch (JSONException e) {
                            Log.e(TAG, "Failed to parse NFT collection response", e);
                            promise.reject("PARSE_ERROR", "Failed to parse response: " + e.getMessage());
                        }
                    }
                });
                
            } catch (Exception e) {
                Log.e(TAG, "Error fetching NFT collection", e);
                promise.reject("FETCH_ERROR", "Error fetching NFT collection: " + e.getMessage());
            }
        });
    }
    
    /**
     * Use Special Card with automatic synergy calculation
     */
    @ReactMethod
    public void useSpecialCard(String cardId, ReadableMap options, Promise promise) {
        if (!isInitialized()) {
            promise.reject("NOT_INITIALIZED", "NFT Service not initialized");
            return;
        }
        
        executorService.execute(() -> {
            try {
                JSONObject requestBody = new JSONObject();
                requestBody.put("cardId", cardId);
                requestBody.put("walletAddress", walletAddress);
                requestBody.put("timestamp", System.currentTimeMillis());
                
                // Add options if provided
                if (options != null) {
                    requestBody.put("targetActivity", options.getString("targetActivity"));
                    requestBody.put("duration", options.getInt("duration"));
                    requestBody.put("autoStack", options.getBoolean("autoStack"));
                }
                
                RequestBody body = RequestBody.create(
                        requestBody.toString(),
                        MediaType.parse("application/json")
                );
                
                Request request = new Request.Builder()
                        .url(CARDS_ENDPOINT + "/use")
                        .addHeader("Authorization", "Bearer " + userToken)
                        .addHeader("Content-Type", "application/json")
                        .post(body)
                        .build();
                
                httpClient.newCall(request).enqueue(new Callback() {
                    @Override
                    public void onFailure(@NonNull Call call, @NonNull IOException e) {
                        Log.e(TAG, "Failed to use special card", e);
                        promise.reject("USE_ERROR", "Failed to use special card: " + e.getMessage());
                    }
                    
                    @Override
                    public void onResponse(@NonNull Call call, @NonNull Response response) throws IOException {
                        try {
                            String responseBody = response.body().string();
                            JSONObject jsonResponse = new JSONObject(responseBody);
                            
                            if (response.isSuccessful()) {
                                WritableMap result = parseCardUsageResult(jsonResponse);
                                promise.resolve(result);
                                
                                // Calculate and emit synergy effects
                                calculateAndEmitSynergyEffects(jsonResponse);
                                
                            } else {
                                promise.reject("API_ERROR", "API Error: " + jsonResponse.optString("message", "Unknown error"));
                            }
                            
                        } catch (JSONException e) {
                            Log.e(TAG, "Failed to parse card usage response", e);
                            promise.reject("PARSE_ERROR", "Failed to parse response: " + e.getMessage());
                        }
                    }
                });
                
            } catch (Exception e) {
                Log.e(TAG, "Error using special card", e);
                promise.reject("USE_ERROR", "Error using special card: " + e.getMessage());
            }
        });
    }
    
    /**
     * Purchase Special Card from marketplace
     */
    @ReactMethod
    public void purchaseSpecialCard(String cardType, String rarity, ReadableMap paymentOptions, Promise promise) {
        if (!isInitialized()) {
            promise.reject("NOT_INITIALIZED", "NFT Service not initialized");
            return;
        }
        
        executorService.execute(() -> {
            try {
                JSONObject requestBody = new JSONObject();
                requestBody.put("cardType", cardType);
                requestBody.put("rarity", rarity);
                requestBody.put("walletAddress", walletAddress);
                requestBody.put("timestamp", System.currentTimeMillis());
                
                // Payment options
                JSONObject payment = new JSONObject();
                payment.put("method", paymentOptions.getString("method")); // "FIN" or "USDFIN"
                payment.put("amount", paymentOptions.getDouble("amount"));
                payment.put("slippage", paymentOptions.optDouble("slippage", 0.5));
                requestBody.put("payment", payment);
                
                RequestBody body = RequestBody.create(
                        requestBody.toString(),
                        MediaType.parse("application/json")
                );
                
                Request request = new Request.Builder()
                        .url(MARKETPLACE_ENDPOINT + "/purchase")
                        .addHeader("Authorization", "Bearer " + userToken)
                        .addHeader("Content-Type", "application/json")
                        .post(body)
                        .build();
                
                httpClient.newCall(request).enqueue(new Callback() {
                    @Override
                    public void onFailure(@NonNull Call call, @NonNull IOException e) {
                        Log.e(TAG, "Failed to purchase special card", e);
                        promise.reject("PURCHASE_ERROR", "Failed to purchase special card: " + e.getMessage());
                    }
                    
                    @Override
                    public void onResponse(@NonNull Call call, @NonNull Response response) throws IOException {
                        try {
                            String responseBody = response.body().string();
                            JSONObject jsonResponse = new JSONObject(responseBody);
                            
                            if (response.isSuccessful()) {
                                WritableMap result = parsePurchaseResult(jsonResponse);
                                promise.resolve(result);
                                
                                // Emit purchase success event
                                emitEvent("cardPurchased", result);
                                
                            } else {
                                String errorMsg = jsonResponse.optString("message", "Purchase failed");
                                promise.reject("PURCHASE_FAILED", errorMsg);
                            }
                            
                        } catch (JSONException e) {
                            Log.e(TAG, "Failed to parse purchase response", e);
                            promise.reject("PARSE_ERROR", "Failed to parse response: " + e.getMessage());
                        }
                    }
                });
                
            } catch (Exception e) {
                Log.e(TAG, "Error purchasing special card", e);
                promise.reject("PURCHASE_ERROR", "Error purchasing special card: " + e.getMessage());
            }
        });
    }
    
    /**
     * Calculate current synergy multiplier from active cards
     */
    @ReactMethod
    public void calculateSynergyMultiplier(Promise promise) {
        if (!isInitialized()) {
            promise.reject("NOT_INITIALIZED", "NFT Service not initialized");
            return;
        }
        
        executorService.execute(() -> {
            try {
                Request request = new Request.Builder()
                        .url(CARDS_ENDPOINT + "/active/" + walletAddress)
                        .addHeader("Authorization", "Bearer " + userToken)
                        .get()
                        .build();
                
                httpClient.newCall(request).enqueue(new Callback() {
                    @Override
                    public void onFailure(@NonNull Call call, @NonNull IOException e) {
                        Log.e(TAG, "Failed to fetch active cards", e);
                        promise.reject("FETCH_ERROR", "Failed to fetch active cards: " + e.getMessage());
                    }
                    
                    @Override
                    public void onResponse(@NonNull Call call, @NonNull Response response) throws IOException {
                        try {
                            String responseBody = response.body().string();
                            JSONObject jsonResponse = new JSONObject(responseBody);
                            
                            if (response.isSuccessful()) {
                                WritableMap synergy = calculateSynergyFromActiveCards(jsonResponse);
                                promise.resolve(synergy);
                                
                            } else {
                                promise.reject("API_ERROR", "API Error: " + jsonResponse.optString("message", "Unknown error"));
                            }
                            
                        } catch (JSONException e) {
                            Log.e(TAG, "Failed to parse active cards response", e);
                            promise.reject("PARSE_ERROR", "Failed to parse response: " + e.getMessage());
                        }
                    }
                });
                
            } catch (Exception e) {
                Log.e(TAG, "Error calculating synergy multiplier", e);
                promise.reject("CALCULATION_ERROR", "Error calculating synergy multiplier: " + e.getMessage());
            }
        });
    }
    
    /**
     * Get marketplace listings with filters
     */
    @ReactMethod
    public void getMarketplaceListings(ReadableMap filters, Promise promise) {
        executorService.execute(() -> {
            try {
                String url = MARKETPLACE_ENDPOINT + "/listings";
                
                // Build query parameters
                StringBuilder queryParams = new StringBuilder("?");
                if (filters != null) {
                    if (filters.hasKey("category")) {
                        queryParams.append("category=").append(filters.getString("category")).append("&");
                    }
                    if (filters.hasKey("rarity")) {
                        queryParams.append("rarity=").append(filters.getString("rarity")).append("&");
                    }
                    if (filters.hasKey("minPrice")) {
                        queryParams.append("minPrice=").append(filters.getDouble("minPrice")).append("&");
                    }
                    if (filters.hasKey("maxPrice")) {
                        queryParams.append("maxPrice=").append(filters.getDouble("maxPrice")).append("&");
                    }
                    if (filters.hasKey("sortBy")) {
                        queryParams.append("sortBy=").append(filters.getString("sortBy")).append("&");
                    }
                }
                
                Request request = new Request.Builder()
                        .url(url + queryParams.toString())
                        .addHeader("Content-Type", "application/json")
                        .get()
                        .build();
                
                httpClient.newCall(request).enqueue(new Callback() {
                    @Override
                    public void onFailure(@NonNull Call call, @NonNull IOException e) {
                        Log.e(TAG, "Failed to fetch marketplace listings", e);
                        promise.reject("FETCH_ERROR", "Failed to fetch marketplace listings: " + e.getMessage());
                    }
                    
                    @Override
                    public void onResponse(@NonNull Call call, @NonNull Response response) throws IOException {
                        try {
                            String responseBody = response.body().string();
                            JSONObject jsonResponse = new JSONObject(responseBody);
                            
                            if (response.isSuccessful()) {
                                WritableArray listings = parseMarketplaceListings(jsonResponse);
                                WritableMap result = Arguments.createMap();
                                result.putArray("listings", listings);
                                result.putInt("total", jsonResponse.optInt("total", 0));
                                result.putInt("page", jsonResponse.optInt("page", 1));
                                
                                promise.resolve(result);
                                
                            } else {
                                promise.reject("API_ERROR", "API Error: " + jsonResponse.optString("message", "Unknown error"));
                            }
                            
                        } catch (JSONException e) {
                            Log.e(TAG, "Failed to parse marketplace response", e);
                            promise.reject("PARSE_ERROR", "Failed to parse response: " + e.getMessage());
                        }
                    }
                });
                
            } catch (Exception e) {
                Log.e(TAG, "Error fetching marketplace listings", e);
                promise.reject("FETCH_ERROR", "Error fetching marketplace listings: " + e.getMessage());
            }
        });
    }
    
    // Helper Methods
    
    private boolean isInitialized() {
        return userToken != null && walletAddress != null;
    }
    
    private WritableMap parseNFTCollection(JSONObject jsonResponse) throws JSONException {
        WritableMap collection = Arguments.createMap();
        
        // Parse Special Cards
        JSONArray specialCards = jsonResponse.optJSONArray("specialCards");
        if (specialCards != null) {
            WritableArray cardsArray = Arguments.createArray();
            for (int i = 0; i < specialCards.length(); i++) {
                JSONObject card = specialCards.getJSONObject(i);
                WritableMap cardMap = parseSpecialCard(card);
                cardsArray.pushMap(cardMap);
            }
            collection.putArray("specialCards", cardsArray);
        }
        
        // Parse Profile Badges
        JSONArray badges = jsonResponse.optJSONArray("profileBadges");
        if (badges != null) {
            WritableArray badgesArray = Arguments.createArray();
            for (int i = 0; i < badges.length(); i++) {
                JSONObject badge = badges.getJSONObject(i);
                WritableMap badgeMap = parseProfileBadge(badge);
                badgesArray.pushMap(badgeMap);
            }
            collection.putArray("profileBadges", badgesArray);
        }
        
        // Parse Achievement NFTs
        JSONArray achievements = jsonResponse.optJSONArray("achievements");
        if (achievements != null) {
            WritableArray achievementsArray = Arguments.createArray();
            for (int i = 0; i < achievements.length(); i++) {
                JSONObject achievement = achievements.getJSONObject(i);
                WritableMap achievementMap = parseAchievementNFT(achievement);
                achievementsArray.pushMap(achievementMap);
            }
            collection.putArray("achievements", achievementsArray);
        }
        
        // Collection statistics
        collection.putInt("totalCards", jsonResponse.optInt("totalCards", 0));
        collection.putInt("totalValue", jsonResponse.optInt("totalValue", 0));
        collection.putDouble("synergyMultiplier", jsonResponse.optDouble("synergyMultiplier", 1.0));
        
        return collection;
    }
    
    private WritableMap parseSpecialCard(JSONObject card) throws JSONException {
        WritableMap cardMap = Arguments.createMap();
        cardMap.putString("id", card.getString("id"));
        cardMap.putString("name", card.getString("name"));
        cardMap.putString("category", card.getString("category"));
        cardMap.putString("rarity", card.getString("rarity"));
        cardMap.putDouble("effect", card.getDouble("effect"));
        cardMap.putInt("duration", card.optInt("duration", 0));
        cardMap.putBoolean("active", card.optBoolean("active", false));
        cardMap.putString("imageUrl", card.optString("imageUrl", ""));
        cardMap.putString("description", card.optString("description", ""));
        cardMap.putInt("usesRemaining", card.optInt("usesRemaining", 1));
        return cardMap;
    }
    
    private WritableMap parseProfileBadge(JSONObject badge) throws JSONException {
        WritableMap badgeMap = Arguments.createMap();
        badgeMap.putString("id", badge.getString("id"));
        badgeMap.putString("name", badge.getString("name"));
        badgeMap.putString("tier", badge.getString("tier"));
        badgeMap.putDouble("miningBonus", badge.getDouble("miningBonus"));
        badgeMap.putDouble("xpBonus", badge.getDouble("xpBonus"));
        badgeMap.putString("imageUrl", badge.optString("imageUrl", ""));
        badgeMap.putBoolean("equipped", badge.optBoolean("equipped", false));
        return badgeMap;
    }
    
    private WritableMap parseAchievementNFT(JSONObject achievement) throws JSONException {
        WritableMap achievementMap = Arguments.createMap();
        achievementMap.putString("id", achievement.getString("id"));
        achievementMap.putString("title", achievement.getString("title"));
        achievementMap.putString("description", achievement.getString("description"));
        achievementMap.putString("category", achievement.getString("category"));
        achievementMap.putString("earnedDate", achievement.getString("earnedDate"));
        achievementMap.putDouble("bonus", achievement.optDouble("bonus", 0.0));
        achievementMap.putString("imageUrl", achievement.optString("imageUrl", ""));
        achievementMap.putBoolean("rare", achievement.optBoolean("rare", false));
        return achievementMap;
    }
    
    private WritableMap parseCardUsageResult(JSONObject response) throws JSONException {
        WritableMap result = Arguments.createMap();
        result.putString("cardId", response.getString("cardId"));
        result.putBoolean("success", response.getBoolean("success"));
        result.putString("transactionHash", response.optString("transactionHash", ""));
        result.putDouble("effectDuration", response.getDouble("effectDuration"));
        result.putDouble("multiplier", response.getDouble("multiplier"));
        result.putDouble("synergyBonus", response.optDouble("synergyBonus", 0.0));
        result.putString("expiresAt", response.getString("expiresAt"));
        
        // Parse active effects
        JSONArray activeEffects = response.optJSONArray("activeEffects");
        if (activeEffects != null) {
            WritableArray effectsArray = Arguments.createArray();
            for (int i = 0; i < activeEffects.length(); i++) {
                JSONObject effect = activeEffects.getJSONObject(i);
                WritableMap effectMap = Arguments.createMap();
                effectMap.putString("type", effect.getString("type"));
                effectMap.putDouble("multiplier", effect.getDouble("multiplier"));
                effectMap.putString("expiresAt", effect.getString("expiresAt"));
                effectsArray.pushMap(effectMap);
            }
            result.putArray("activeEffects", effectsArray);
        }
        
        return result;
    }
    
    private WritableMap parsePurchaseResult(JSONObject response) throws JSONException {
        WritableMap result = Arguments.createMap();
        result.putString("transactionId", response.getString("transactionId"));
        result.putString("cardId", response.getString("cardId"));
        result.putString("cardName", response.getString("cardName"));
        result.putString("rarity", response.getString("rarity"));
        result.putDouble("pricePaid", response.getDouble("pricePaid"));
        result.putString("currency", response.getString("currency"));
        result.putString("purchaseDate", response.getString("purchaseDate"));
        result.putBoolean("success", response.getBoolean("success"));
        return result;
    }
    
    private WritableMap calculateSynergyFromActiveCards(JSONObject response) throws JSONException {
        WritableMap synergy = Arguments.createMap();
        
        JSONArray activeCards = response.getJSONArray("activeCards");
        int cardCount = activeCards.length();
        
        // Base synergy calculation: 1.0 + (cardCount * 0.1)
        double baseSynergy = 1.0 + (cardCount * 0.1);
        
        // Calculate rarity bonus
        double rarityBonus = 0.0;
        Map<String, Integer> categoryCount = new HashMap<>();
        
        for (int i = 0; i < activeCards.length(); i++) {
            JSONObject card = activeCards.getJSONObject(i);
            String rarity = card.getString("rarity");
            String category = card.getString("category");
            
            // Add rarity bonus
            switch (rarity.toLowerCase()) {
                case "uncommon": rarityBonus += 0.05; break;
                case "rare": rarityBonus += 0.10; break;
                case "epic": rarityBonus += 0.20; break;
                case "legendary": rarityBonus += 0.35; break;
            }
            
            // Count categories
            categoryCount.put(category, categoryCount.getOrDefault(category, 0) + 1);
        }
        
        // Type match bonus
        double typeMatchBonus = 0.0;
        boolean hasAllCategories = categoryCount.size() >= 3;
        if (hasAllCategories) {
            typeMatchBonus = 0.30; // All three categories active
        } else if (categoryCount.size() >= 2) {
            typeMatchBonus = 0.15; // Same category cards active
        }
        
        double finalMultiplier = baseSynergy + rarityBonus + typeMatchBonus;
        
        synergy.putDouble("multiplier", finalMultiplier);
        synergy.putInt("activeCards", cardCount);
        synergy.putDouble("rarityBonus", rarityBonus);
        synergy.putDouble("typeMatchBonus", typeMatchBonus);
        synergy.putMap("categoryBreakdown", convertMapToWritableMap(categoryCount));
        
        return synergy;
    }
    
    private WritableArray parseMarketplaceListings(JSONObject response) throws JSONException {
        WritableArray listings = Arguments.createArray();
        JSONArray listingsArray = response.getJSONArray("listings");
        
        for (int i = 0; i < listingsArray.length(); i++) {
            JSONObject listing = listingsArray.getJSONObject(i);
            WritableMap listingMap = Arguments.createMap();
            
            listingMap.putString("id", listing.getString("id"));
            listingMap.putString("cardName", listing.getString("cardName"));
            listingMap.putString("category", listing.getString("category"));
            listingMap.putString("rarity", listing.getString("rarity"));
            listingMap.putDouble("price", listing.getDouble("price"));
            listingMap.putString("currency", listing.getString("currency"));
            listingMap.putString("seller", listing.getString("seller"));
            listingMap.putString("listedDate", listing.getString("listedDate"));
            listingMap.putString("imageUrl", listing.optString("imageUrl", ""));
            listingMap.putDouble("effect", listing.getDouble("effect"));
            listingMap.putInt("duration", listing.optInt("duration", 0));
            
                listings.pushMap(listingMap);
        }
        
        return listings;
    }
    
    private void calculateAndEmitSynergyEffects(JSONObject response) {
        try {
            // Calculate current synergy after card usage
            JSONArray activeCards = response.optJSONArray("activeCards");
            if (activeCards != null) {
                WritableMap synergyData = calculateSynergyFromActiveCards(response);
                emitEvent("synergyEffectUpdated", synergyData);
            }
        } catch (Exception e) {
            Log.e(TAG, "Error calculating synergy effects", e);
        }
    }
    
    private WritableMap convertMapToWritableMap(Map<String, Integer> map) {
        WritableMap writableMap = Arguments.createMap();
        for (Map.Entry<String, Integer> entry : map.entrySet()) {
            writableMap.putInt(entry.getKey(), entry.getValue());
        }
        return writableMap;
    }
    
    private void emitEvent(String eventName, @Nullable WritableMap params) {
        if (reactContext.hasActiveCatalystInstance()) {
            reactContext
                .getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter.class)
                .emit(eventName, params);
        }
    }
    
    /**
     * Get detailed card information including utility bonuses
     */
    @ReactMethod
    public void getCardDetails(String cardId, Promise promise) {
        if (!isInitialized()) {
            promise.reject("NOT_INITIALIZED", "NFT Service not initialized");
            return;
        }
        
        executorService.execute(() -> {
            try {
                Request request = new Request.Builder()
                        .url(CARDS_ENDPOINT + "/details/" + cardId)
                        .addHeader("Authorization", "Bearer " + userToken)
                        .get()
                        .build();
                
                httpClient.newCall(request).enqueue(new Callback() {
                    @Override
                    public void onFailure(@NonNull Call call, @NonNull IOException e) {
                        Log.e(TAG, "Failed to fetch card details", e);
                        promise.reject("FETCH_ERROR", "Failed to fetch card details: " + e.getMessage());
                    }
                    
                    @Override
                    public void onResponse(@NonNull Call call, @NonNull Response response) throws IOException {
                        try {
                            String responseBody = response.body().string();
                            JSONObject jsonResponse = new JSONObject(responseBody);
                            
                            if (response.isSuccessful()) {
                                WritableMap cardDetails = parseDetailedCardInfo(jsonResponse);
                                promise.resolve(cardDetails);
                                
                            } else {
                                promise.reject("API_ERROR", "API Error: " + jsonResponse.optString("message", "Unknown error"));
                            }
                            
                        } catch (JSONException e) {
                            Log.e(TAG, "Failed to parse card details response", e);
                            promise.reject("PARSE_ERROR", "Failed to parse response: " + e.getMessage());
                        }
                    }
                });
                
            } catch (Exception e) {
                Log.e(TAG, "Error fetching card details", e);
                promise.reject("FETCH_ERROR", "Error fetching card details: " + e.getMessage());
            }
        });
    }
    
    /**
     * Mint Achievement NFT when user reaches milestone
     */
    @ReactMethod
    public void mintAchievementNFT(String achievementType, ReadableMap achievementData, Promise promise) {
        if (!isInitialized()) {
            promise.reject("NOT_INITIALIZED", "NFT Service not initialized");
            return;
        }
        
        executorService.execute(() -> {
            try {
                JSONObject requestBody = new JSONObject();
                requestBody.put("achievementType", achievementType);
                requestBody.put("walletAddress", walletAddress);
                requestBody.put("timestamp", System.currentTimeMillis());
                
                // Add achievement-specific data
                JSONObject achievementMetadata = new JSONObject();
                if (achievementData.hasKey("level")) {
                    achievementMetadata.put("level", achievementData.getInt("level"));
                }
                if (achievementData.hasKey("milestone")) {
                    achievementMetadata.put("milestone", achievementData.getString("milestone"));
                }
                if (achievementData.hasKey("value")) {
                    achievementMetadata.put("value", achievementData.getDouble("value"));
                }
                if (achievementData.hasKey("date")) {
                    achievementMetadata.put("date", achievementData.getString("date"));
                }
                requestBody.put("metadata", achievementMetadata);
                
                RequestBody body = RequestBody.create(
                        requestBody.toString(),
                        MediaType.parse("application/json")
                );
                
                Request request = new Request.Builder()
                        .url(NFT_ENDPOINT + "/mint-achievement")
                        .addHeader("Authorization", "Bearer " + userToken)
                        .addHeader("Content-Type", "application/json")
                        .post(body)
                        .build();
                
                httpClient.newCall(request).enqueue(new Callback() {
                    @Override
                    public void onFailure(@NonNull Call call, @NonNull IOException e) {
                        Log.e(TAG, "Failed to mint achievement NFT", e);
                        promise.reject("MINT_ERROR", "Failed to mint achievement NFT: " + e.getMessage());
                    }
                    
                    @Override
                    public void onResponse(@NonNull Call call, @NonNull Response response) throws IOException {
                        try {
                            String responseBody = response.body().string();
                            JSONObject jsonResponse = new JSONObject(responseBody);
                            
                            if (response.isSuccessful()) {
                                WritableMap result = parseMintResult(jsonResponse);
                                promise.resolve(result);
                                
                                // Emit achievement earned event
                                emitEvent("achievementEarned", result);
                                
                            } else {
                                String errorMsg = jsonResponse.optString("message", "Minting failed");
                                promise.reject("MINT_FAILED", errorMsg);
                            }
                            
                        } catch (JSONException e) {
                            Log.e(TAG, "Failed to parse mint response", e);
                            promise.reject("PARSE_ERROR", "Failed to parse response: " + e.getMessage());
                        }
                    }
                });
                
            } catch (Exception e) {
                Log.e(TAG, "Error minting achievement NFT", e);
                promise.reject("MINT_ERROR", "Error minting achievement NFT: " + e.getMessage());
            }
        });
    }
    
    /**
     * Transfer NFT to another user (marketplace or P2P)
     */
    @ReactMethod
    public void transferNFT(String nftId, String recipientAddress, ReadableMap options, Promise promise) {
        if (!isInitialized()) {
            promise.reject("NOT_INITIALIZED", "NFT Service not initialized");
            return;
        }
        
        executorService.execute(() -> {
            try {
                JSONObject requestBody = new JSONObject();
                requestBody.put("nftId", nftId);
                requestBody.put("fromAddress", walletAddress);
                requestBody.put("toAddress", recipientAddress);
                requestBody.put("timestamp", System.currentTimeMillis());
                
                // Transfer options
                if (options != null) {
                    if (options.hasKey("price")) {
                        requestBody.put("price", options.getDouble("price"));
                    }
                    if (options.hasKey("currency")) {
                        requestBody.put("currency", options.getString("currency"));
                    }
                    if (options.hasKey("marketplaceListing")) {
                        requestBody.put("isMarketplace", options.getBoolean("marketplaceListing"));
                    }
                }
                
                RequestBody body = RequestBody.create(
                        requestBody.toString(),
                        MediaType.parse("application/json")
                );
                
                Request request = new Request.Builder()
                        .url(NFT_ENDPOINT + "/transfer")
                        .addHeader("Authorization", "Bearer " + userToken)
                        .addHeader("Content-Type", "application/json")
                        .post(body)
                        .build();
                
                httpClient.newCall(request).enqueue(new Callback() {
                    @Override
                    public void onFailure(@NonNull Call call, @NonNull IOException e) {
                        Log.e(TAG, "Failed to transfer NFT", e);
                        promise.reject("TRANSFER_ERROR", "Failed to transfer NFT: " + e.getMessage());
                    }
                    
                    @Override
                    public void onResponse(@NonNull Call call, @NonNull Response response) throws IOException {
                        try {
                            String responseBody = response.body().string();
                            JSONObject jsonResponse = new JSONObject(responseBody);
                            
                            if (response.isSuccessful()) {
                                WritableMap result = parseTransferResult(jsonResponse);
                                promise.resolve(result);
                                
                                // Emit transfer event
                                emitEvent("nftTransferred", result);
                                
                            } else {
                                String errorMsg = jsonResponse.optString("message", "Transfer failed");
                                promise.reject("TRANSFER_FAILED", errorMsg);
                            }
                            
                        } catch (JSONException e) {
                            Log.e(TAG, "Failed to parse transfer response", e);
                            promise.reject("PARSE_ERROR", "Failed to parse response: " + e.getMessage());
                        }
                    }
                });
                
            } catch (Exception e) {
                Log.e(TAG, "Error transferring NFT", e);
                promise.reject("TRANSFER_ERROR", "Error transferring NFT: " + e.getMessage());
            }
        });
    }
    
    /**
     * Upgrade profile badge to next tier
     */
    @ReactMethod
    public void upgradeProfileBadge(String badgeId, ReadableMap upgradeOptions, Promise promise) {
        if (!isInitialized()) {
            promise.reject("NOT_INITIALIZED", "NFT Service not initialized");
            return;
        }
        
        executorService.execute(() -> {
            try {
                JSONObject requestBody = new JSONObject();
                requestBody.put("badgeId", badgeId);
                requestBody.put("walletAddress", walletAddress);
                requestBody.put("timestamp", System.currentTimeMillis());
                
                // Upgrade requirements
                if (upgradeOptions != null) {
                    if (upgradeOptions.hasKey("burnCards")) {
                        // Convert ReadableArray to JSONArray for cards to burn
                        requestBody.put("burnCards", upgradeOptions.getArray("burnCards"));
                    }
                    if (upgradeOptions.hasKey("paymentAmount")) {
                        requestBody.put("paymentAmount", upgradeOptions.getDouble("paymentAmount"));
                    }
                    if (upgradeOptions.hasKey("paymentCurrency")) {
                        requestBody.put("paymentCurrency", upgradeOptions.getString("paymentCurrency"));
                    }
                }
                
                RequestBody body = RequestBody.create(
                        requestBody.toString(),
                        MediaType.parse("application/json")
                );
                
                Request request = new Request.Builder()
                        .url(NFT_ENDPOINT + "/upgrade-badge")
                        .addHeader("Authorization", "Bearer " + userToken)
                        .addHeader("Content-Type", "application/json")
                        .post(body)
                        .build();
                
                httpClient.newCall(request).enqueue(new Callback() {
                    @Override
                    public void onFailure(@NonNull Call call, @NonNull IOException e) {
                        Log.e(TAG, "Failed to upgrade badge", e);
                        promise.reject("UPGRADE_ERROR", "Failed to upgrade badge: " + e.getMessage());
                    }
                    
                    @Override
                    public void onResponse(@NonNull Call call, @NonNull Response response) throws IOException {
                        try {
                            String responseBody = response.body().string();
                            JSONObject jsonResponse = new JSONObject(responseBody);
                            
                            if (response.isSuccessful()) {
                                WritableMap result = parseUpgradeResult(jsonResponse);
                                promise.resolve(result);
                                
                                // Emit badge upgrade event
                                emitEvent("badgeUpgraded", result);
                                
                            } else {
                                String errorMsg = jsonResponse.optString("message", "Upgrade failed");
                                promise.reject("UPGRADE_FAILED", errorMsg);
                            }
                            
                        } catch (JSONException e) {
                            Log.e(TAG, "Failed to parse upgrade response", e);
                            promise.reject("PARSE_ERROR", "Failed to parse response: " + e.getMessage());
                        }
                    }
                });
                
            } catch (Exception e) {
                Log.e(TAG, "Error upgrading badge", e);
                promise.reject("UPGRADE_ERROR", "Error upgrading badge: " + e.getMessage());
            }
        });
    }
    
    // Additional parsing helper methods
    
    private WritableMap parseDetailedCardInfo(JSONObject response) throws JSONException {
        WritableMap cardInfo = Arguments.createMap();
        
        // Basic card info
        cardInfo.putString("id", response.getString("id"));
        cardInfo.putString("name", response.getString("name"));
        cardInfo.putString("description", response.getString("description"));
        cardInfo.putString("category", response.getString("category"));
        cardInfo.putString("rarity", response.getString("rarity"));
        cardInfo.putString("imageUrl", response.optString("imageUrl", ""));
        
        // Effect details
        WritableMap effects = Arguments.createMap();
        JSONObject effectsJson = response.getJSONObject("effects");
        effects.putDouble("multiplier", effectsJson.getDouble("multiplier"));
        effects.putInt("durationHours", effectsJson.getInt("durationHours"));
        effects.putString("targetActivity", effectsJson.getString("targetActivity"));
        effects.putBoolean("stackable", effectsJson.optBoolean("stackable", false));
        cardInfo.putMap("effects", effects);
        
        // Market data
        WritableMap marketData = Arguments.createMap();
        JSONObject marketJson = response.optJSONObject("marketData");
        if (marketJson != null) {
            marketData.putDouble("currentPrice", marketJson.optDouble("currentPrice", 0.0));
            marketData.putDouble("averagePrice", marketJson.optDouble("averagePrice", 0.0));
            marketData.putInt("totalSupply", marketJson.optInt("totalSupply", 0));
            marketData.putInt("circulatingSupply", marketJson.optInt("circulatingSupply", 0));
        }
        cardInfo.putMap("marketData", marketData);
        
        // Usage statistics
        WritableMap usage = Arguments.createMap();
        JSONObject usageJson = response.optJSONObject("usageStats");
        if (usageJson != null) {
            usage.putInt("timesUsed", usageJson.optInt("timesUsed", 0));
            usage.putString("lastUsed", usageJson.optString("lastUsed", ""));
            usage.putDouble("totalEffectTime", usageJson.optDouble("totalEffectTime", 0.0));
        }
        cardInfo.putMap("usageStats", usage);
        
        return cardInfo;
    }
    
    private WritableMap parseMintResult(JSONObject response) throws JSONException {
        WritableMap result = Arguments.createMap();
        result.putString("nftId", response.getString("nftId"));
        result.putString("achievementType", response.getString("achievementType"));
        result.putString("transactionHash", response.getString("transactionHash"));
        result.putString("mintDate", response.getString("mintDate"));
        result.putDouble("bonusEffect", response.optDouble("bonusEffect", 0.0));
        result.putString("imageUrl", response.optString("imageUrl", ""));
        result.putBoolean("isRare", response.optBoolean("isRare", false));
        return result;
    }
    
    private WritableMap parseTransferResult(JSONObject response) throws JSONException {
        WritableMap result = Arguments.createMap();
        result.putString("transferId", response.getString("transferId"));
        result.putString("nftId", response.getString("nftId"));
        result.putString("fromAddress", response.getString("fromAddress"));
        result.putString("toAddress", response.getString("toAddress"));
        result.putString("transactionHash", response.getString("transactionHash"));
        result.putString("transferDate", response.getString("transferDate"));
        result.putDouble("price", response.optDouble("price", 0.0));
        result.putString("currency", response.optString("currency", ""));
        result.putBoolean("success", response.getBoolean("success"));
        return result;
    }
    
    private WritableMap parseUpgradeResult(JSONObject response) throws JSONException {
        WritableMap result = Arguments.createMap();
        result.putString("badgeId", response.getString("badgeId"));
        result.putString("oldTier", response.getString("oldTier"));
        result.putString("newTier", response.getString("newTier"));
        result.putString("transactionHash", response.getString("transactionHash"));
        result.putString("upgradeDate", response.getString("upgradeDate"));
        result.putDouble("newMiningBonus", response.getDouble("newMiningBonus"));
        result.putDouble("newXPBonus", response.getDouble("newXPBonus"));
        result.putDouble("upgradeCost", response.optDouble("upgradeCost", 0.0));
        result.putBoolean("success", response.getBoolean("success"));
        return result;
    }
    
    /**
     * Clean up resources
     */
    @ReactMethod
    public void cleanup() {
        Log.d(TAG, "Cleaning up NFT Service resources");
        
        if (executorService != null && !executorService.isShutdown()) {
            executorService.shutdown();
            try {
                if (!executorService.awaitTermination(5, java.util.concurrent.TimeUnit.SECONDS)) {
                    executorService.shutdownNow();
                }
            } catch (InterruptedException e) {
                executorService.shutdownNow();
                Thread.currentThread().interrupt();
            }
        }
        
        // Clear sensitive data
        userToken = null;
        walletAddress = null;
    }
    
    @Override
    public void onCatalystInstanceDestroy() {
        super.onCatalystInstanceDestroy();
        cleanup();
    }
}
