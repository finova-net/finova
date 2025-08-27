package com.finova.reactnative;

import com.facebook.react.bridge.*;
import com.facebook.react.modules.core.DeviceEventManagerModule;
import android.content.SharedPreferences;
import android.content.Context;
import android.util.Log;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * NFT Module - Special Cards and NFT management
 */
@ReactModule(name = NFTModule.NAME)
public class NFTModule extends ReactContextBaseJavaModule {
    public static final String NAME = "FinovaNFT";
    private static final String TAG = "NFTModule";
    private final ReactApplicationContext reactContext;
    private final SharedPreferences prefs;
    private final ExecutorService executor;

    public NFTModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.reactContext = reactContext;
        this.prefs = reactContext.getSharedPreferences("finova_nft", Context.MODE_PRIVATE);
        this.executor = Executors.newFixedThreadPool(2);
    }

    @Override
    public String getName() {
        return NAME;
    }

    @ReactMethod
    public void getOwnedNFTs(Promise promise) {
        executor.execute(() -> {
            try {
                WritableArray nfts = Arguments.createArray();
                
                // Mock NFT data - replace with actual blockchain query
                String[] cardTypes = {"DoubleMining", "XPDouble", "ReferralBoost", "MiningFrenzy"};
                String[] rarities = {"Common", "Uncommon", "Rare", "Epic", "Legendary"};
                
                for (int i = 0; i < 8; i++) {
                    WritableMap nft = Arguments.createMap();
                    nft.putString("tokenId", "nft_" + (1000 + i));
                    nft.putString("name", cardTypes[i % cardTypes.length]);
                    nft.putString("rarity", rarities[i % rarities.length]);
                    nft.putString("category", i < 2 ? "mining" : i < 4 ? "xp" : "referral");
                    nft.putBoolean("isActive", i < 2);
                    nft.putLong("expiresAt", i < 2 ? System.currentTimeMillis() + (24 * 60 * 60 * 1000) : 0);
                    nft.putString("imageUrl", "https://api.finova.network/nft/image/" + (1000 + i));
                    nft.putString("description", "Special card with unique abilities");
                    nft.putInt("uses", i % 2 == 0 ? -1 : 3 - (i % 3)); // -1 for unlimited
                    nfts.pushMap(nft);
                }

                promise.resolve(nfts);

            } catch (Exception e) {
                Log.e(TAG, "Failed to get owned NFTs", e);
                promise.reject("NFT_FETCH_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void useSpecialCard(String tokenId, Promise promise) {
        executor.execute(() -> {
            try {
                // Validate card ownership and usage
                if (!isCardOwned(tokenId)) {
                    promise.reject("CARD_NOT_OWNED", "Card not found in user inventory");
                    return;
                }

                if (isCardActive(tokenId)) {
                    promise.reject("CARD_ALREADY_ACTIVE", "Card is already active");
                    return;
                }

                String cardType = getCardType(tokenId);
                long duration = getCardDuration(cardType);
                double effect = getCardEffect(cardType);

                // Activate card
                SharedPreferences.Editor editor = prefs.edit();
                editor.putLong("card_" + tokenId + "_activated", System.currentTimeMillis());
                editor.putLong("card_" + tokenId + "_expires", System.currentTimeMillis() + duration);
                editor.putString("active_card_" + tokenId, cardType);
                editor.apply();

                // Decrease uses for limited-use cards
                decreaseCardUses(tokenId);

                WritableMap response = Arguments.createMap();
                response.putBoolean("success", true);
                response.putString("cardType", cardType);
                response.putDouble("effect", effect);
                response.putLong("duration", duration);
                response.putLong("expiresAt", System.currentTimeMillis() + duration);
                response.putString("message", "Special card activated successfully!");

                promise.resolve(response);
                sendEvent("onCardActivated", response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to use special card", e);
                promise.reject("CARD_USE_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void getActiveCards(Promise promise) {
        executor.execute(() -> {
            try {
                WritableArray activeCards = Arguments.createArray();
                long currentTime = System.currentTimeMillis();

                // Mock active cards check
                for (int i = 1000; i < 1008; i++) {
                    String tokenId = "nft_" + i;
                    long expiresAt = prefs.getLong("card_" + tokenId + "_expires", 0);
                    
                    if (expiresAt > currentTime) {
                        WritableMap card = Arguments.createMap();
                        card.putString("tokenId", tokenId);
                        card.putString("type", getCardType(tokenId));
                        card.putLong("activatedAt", prefs.getLong("card_" + tokenId + "_activated", 0));
                        card.putLong("expiresAt", expiresAt);
                        card.putLong("remainingTime", expiresAt - currentTime);
                        card.putDouble("effect", getCardEffect(getCardType(tokenId)));
                        activeCards.pushMap(card);
                    }
                }

                promise.resolve(activeCards);

            } catch (Exception e) {
                Log.e(TAG, "Failed to get active cards", e);
                promise.reject("ACTIVE_CARDS_FETCH_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void purchaseCard(String cardType, String rarity, Promise promise) {
        executor.execute(() -> {
            try {
                double price = getCardPrice(cardType, rarity);
                double userBalance = getUserBalance();

                if (userBalance < price) {
                    promise.reject("INSUFFICIENT_FUNDS", "Not enough FIN tokens to purchase card");
                    return;
                }

                // Mock purchase process
                String tokenId = "nft_" + System.currentTimeMillis();
                
                // Deduct balance
                SharedPreferences.Editor editor = prefs.edit();
                editor.putFloat("user_balance", (float)(userBalance - price));
                editor.putString("owned_card_" + tokenId, cardType + ":" + rarity);
                editor.apply();

                WritableMap response = Arguments.createMap();
                response.putBoolean("success", true);
                response.putString("tokenId", tokenId);
                response.putString("cardType", cardType);
                response.putString("rarity", rarity);
                response.putDouble("price", price);
                response.putDouble("remainingBalance", userBalance - price);
                response.putString("transactionHash", "tx_" + tokenId);

                promise.resolve(response);
                sendEvent("onCardPurchased", response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to purchase card", e);
                promise.reject("CARD_PURCHASE_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void getMarketplace(Promise promise) {
        executor.execute(() -> {
            try {
                WritableArray marketplace = Arguments.createArray();
                
                String[] cardTypes = {"DoubleMining", "TripleMining", "XPDouble", "ReferralBoost", "MiningFrenzy"};
                String[] rarities = {"Common", "Rare", "Epic", "Legendary"};
                
                for (String cardType : cardTypes) {
                    for (String rarity : rarities) {
                        WritableMap item = Arguments.createMap();
                        item.putString("cardType", cardType);
                        item.putString("rarity", rarity);
                        item.putDouble("price", getCardPrice(cardType, rarity));
                        item.putDouble("effect", getCardEffect(cardType));
                        item.putLong("duration", getCardDuration(cardType));
                        item.putString("description", getCardDescription(cardType));
                        item.putString("imageUrl", "https://api.finova.network/cards/" + cardType.toLowerCase() + ".png");
                        item.putBoolean("available", true);
                        marketplace.pushMap(item);
                    }
                }

                promise.resolve(marketplace);

            } catch (Exception e) {
                Log.e(TAG, "Failed to get marketplace", e);
                promise.reject("MARKETPLACE_FETCH_FAILED", e.getMessage());
            }
        });
    }

    // Helper methods for NFT operations
    private boolean isCardOwned(String tokenId) {
        return prefs.contains("owned_card_" + tokenId);
    }

    private boolean isCardActive(String tokenId) {
        long expiresAt = prefs.getLong("card_" + tokenId + "_expires", 0);
        return expiresAt > System.currentTimeMillis();
    }

    private String getCardType(String tokenId) {
        String cardData = prefs.getString("owned_card_" + tokenId, "");
        return cardData.split(":")[0];
    }

    private double getCardPrice(String cardType, String rarity) {
        double basePrice = cardType.equals("DoubleMining") ? 50 :
                          cardType.equals("TripleMining") ? 150 :
                          cardType.equals("XPDouble") ? 40 :
                          cardType.equals("ReferralBoost") ? 60 : 100;
        
        double rarityMultiplier = rarity.equals("Common") ? 1.0 :
                                 rarity.equals("Rare") ? 2.5 :
                                 rarity.equals("Epic") ? 5.0 : 10.0;
        
        return basePrice * rarityMultiplier;
    }

    private double getCardEffect(String cardType) {
        switch (cardType) {
            case "DoubleMining": return 2.0;
            case "TripleMining": return 3.0;
            case "MiningFrenzy": return 6.0;
            case "XPDouble": return 2.0;
            case "ReferralBoost": return 1.5;
            default: return 1.0;
        }
    }

    private long getCardDuration(String cardType) {
        switch (cardType) {
            case "DoubleMining": return 24 * 60 * 60 * 1000L; // 24 hours
            case "TripleMining": return 12 * 60 * 60 * 1000L; // 12 hours
            case "MiningFrenzy": return 4 * 60 * 60 * 1000L;  // 4 hours
            case "XPDouble": return 24 * 60 * 60 * 1000L;     // 24 hours
            case "ReferralBoost": return 7 * 24 * 60 * 60 * 1000L; // 7 days
            default: return 60 * 60 * 1000L; // 1 hour
        }
    }

    private String getCardDescription(String cardType) {
        switch (cardType) {
            case "DoubleMining": return "Double your mining rate for 24 hours";
            case "TripleMining": return "Triple your mining rate for 12 hours";
            case "MiningFrenzy": return "5x mining rate for 4 hours";
            case "XPDouble": return "Double XP from all activities for 24 hours";
            case "ReferralBoost": return "50% bonus on referral rewards for 7 days";
            default: return "Special enhancement card";
        }
    }

    private void decreaseCardUses(String tokenId) {
        // Implementation for limited-use cards
    }

    private double getUserBalance() {
        return prefs.getFloat("user_balance", 100.0f);
    }

    private void sendEvent(String eventName, WritableMap data) {
        if (reactContext.hasActiveCatalystInstance()) {
            reactContext.getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter.class)
                .emit(eventName, data);
        }
    }
}

/**
 * Wallet Module - Crypto wallet operations and e-wallet integration
 */
@ReactModule(name = WalletModule.NAME)
public class WalletModule extends ReactContextBaseJavaModule {
    public static final String NAME = "FinovaWallet";
    private static final String TAG = "WalletModule";
    private final ReactApplicationContext reactContext;
    private final SharedPreferences prefs;
    private final ExecutorService executor;

    public WalletModule(ReactApplicationContext reactContext) {
        super(reactContext);
        this.reactContext = reactContext;
        this.prefs = reactContext.getSharedPreferences("finova_wallet", Context.MODE_PRIVATE);
        this.executor = Executors.newFixedThreadPool(2);
    }

    @Override
    public String getName() {
        return NAME;
    }

    @ReactMethod
    public void getBalance(Promise promise) {
        executor.execute(() -> {
            try {
                WritableMap balance = Arguments.createMap();
                balance.putDouble("fin", prefs.getFloat("fin_balance", 0.0f));
                balance.putDouble("sfin", prefs.getFloat("sfin_balance", 0.0f));
                balance.putDouble("usdfin", prefs.getFloat("usdfin_balance", 0.0f));
                balance.putDouble("susdfin", prefs.getFloat("susdfin_balance", 0.0f));
                balance.putString("walletAddress", prefs.getString("wallet_address", ""));
                balance.putLong("lastUpdated", System.currentTimeMillis());

                promise.resolve(balance);

            } catch (Exception e) {
                Log.e(TAG, "Failed to get balance", e);
                promise.reject("BALANCE_FETCH_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void sendTransaction(ReadableMap txData, Promise promise) {
        executor.execute(() -> {
            try {
                String recipient = txData.getString("recipient");
                double amount = txData.getDouble("amount");
                String tokenType = txData.hasKey("tokenType") ? txData.getString("tokenType") : "fin";

                // Validate transaction
                if (recipient == null || recipient.isEmpty()) {
                    promise.reject("INVALID_RECIPIENT", "Recipient address is required");
                    return;
                }

                if (amount <= 0) {
                    promise.reject("INVALID_AMOUNT", "Amount must be greater than 0");
                    return;
                }

                double currentBalance = getCurrentBalance(tokenType);
                if (amount > currentBalance) {
                    promise.reject("INSUFFICIENT_FUNDS", "Not enough " + tokenType.toUpperCase() + " tokens");
                    return;
                }

                // Mock transaction processing
                String txHash = "tx_" + System.currentTimeMillis();
                Thread.sleep(2000); // Simulate network delay

                // Update balance
                updateBalance(tokenType, currentBalance - amount);

                WritableMap response = Arguments.createMap();
                response.putBoolean("success", true);
                response.putString("transactionHash", txHash);
                response.putString("recipient", recipient);
                response.putDouble("amount", amount);
                response.putString("tokenType", tokenType);
                response.putDouble("newBalance", currentBalance - amount);
                response.putLong("timestamp", System.currentTimeMillis());

                promise.resolve(response);
                sendEvent("onTransactionSent", response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to send transaction", e);
                promise.reject("TRANSACTION_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void getTransactionHistory(int limit, Promise promise) {
        executor.execute(() -> {
            try {
                WritableArray history = Arguments.createArray();

                // Mock transaction history
                String[] types = {"mining", "referral", "purchase", "transfer", "stake"};
                for (int i = 0; i < Math.min(limit, 20); i++) {
                    WritableMap tx = Arguments.createMap();
                    tx.putString("hash", "tx_" + (System.currentTimeMillis() - (i * 60000)));
                    tx.putString("type", types[i % types.length]);
                    tx.putDouble("amount", 1.5 + (Math.random() * 10));
                    tx.putString("tokenType", i % 3 == 0 ? "fin" : i % 3 == 1 ? "sfin" : "usdfin");
                    tx.putLong("timestamp", System.currentTimeMillis() - (i * 60000L));
                    tx.putString("status", "confirmed");
                    tx.putString("from", i % 2 == 0 ? prefs.getString("wallet_address", "") : "system");
                    tx.putString("to", i % 2 == 0 ? "system" : prefs.getString("wallet_address", ""));
                    history.pushMap(tx);
                }

                promise.resolve(history);

            } catch (Exception e) {
                Log.e(TAG, "Failed to get transaction history", e);
                promise.reject("HISTORY_FETCH_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void stakeTokens(ReadableMap stakeData, Promise promise) {
        executor.execute(() -> {
            try {
                double amount = stakeData.getDouble("amount");
                int duration = stakeData.hasKey("duration") ? stakeData.getInt("duration") : 30; // days

                double finBalance = getCurrentBalance("fin");
                if (amount > finBalance) {
                    promise.reject("INSUFFICIENT_FUNDS", "Not enough FIN tokens to stake");
                    return;
                }

                // Calculate staking rewards
                double apy = calculateStakingAPY(amount);
                long unstakeTime = System.currentTimeMillis() + (duration * 24L * 60 * 60 * 1000);

                // Update balances
                updateBalance("fin", finBalance - amount);
                updateBalance("sfin", getCurrentBalance("sfin") + amount);

                // Store staking info
                SharedPreferences.Editor editor = prefs.edit();
                editor.putFloat("staked_amount", (float)amount);
                editor.putLong("stake_time", System.currentTimeMillis());
                editor.putLong("unstake_time", unstakeTime);
                editor.putFloat("staking_apy", (float)apy);
                editor.apply();

                WritableMap response = Arguments.createMap();
                response.putBoolean("success", true);
                response.putDouble("stakedAmount", amount);
                response.putDouble("apy", apy);
                response.putLong("unstakeTime", unstakeTime);
                response.putDouble("estimatedRewards", amount * apy / 100 * duration / 365);
                response.putString("message", "Tokens staked successfully!");

                promise.resolve(response);
                sendEvent("onTokensStaked", response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to stake tokens", e);
                promise.reject("STAKING_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void unstakeTokens(Promise promise) {
        executor.execute(() -> {
            try {
                double stakedAmount = prefs.getFloat("staked_amount", 0.0f);
                long unstakeTime = prefs.getLong("unstake_time", 0);
                long currentTime = System.currentTimeMillis();

                if (stakedAmount == 0) {
                    promise.reject("NO_STAKED_TOKENS", "No tokens currently staked");
                    return;
                }

                if (currentTime < unstakeTime) {
                    promise.reject("STAKING_LOCKED", "Tokens are still locked in staking period");
                    return;
                }

                // Calculate rewards
                long stakeTime = prefs.getLong("stake_time", 0);
                double stakingDays = (currentTime - stakeTime) / (24.0 * 60 * 60 * 1000);
                double apy = prefs.getFloat("staking_apy", 0.0f);
                double rewards = stakedAmount * apy / 100 * stakingDays / 365;

                // Update balances
                updateBalance("fin", getCurrentBalance("fin") + stakedAmount + rewards);
                updateBalance("sfin", getCurrentBalance("sfin") - stakedAmount);

                // Clear staking data
                SharedPreferences.Editor editor = prefs.edit();
                editor.remove("staked_amount");
                editor.remove("stake_time");
                editor.remove("unstake_time");
                editor.remove("staking_apy");
                editor.apply();

                WritableMap response = Arguments.createMap();
                response.putBoolean("success", true);
                response.putDouble("unstakedAmount", stakedAmount);
                response.putDouble("rewards", rewards);
                response.putDouble("totalReceived", stakedAmount + rewards);
                response.putString("message", "Tokens unstaked successfully!");

                promise.resolve(response);
                sendEvent("onTokensUnstaked", response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to unstake tokens", e);
                promise.reject("UNSTAKING_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void getStakingInfo(Promise promise) {
        try {
            WritableMap stakingInfo = Arguments.createMap();
            double stakedAmount = prefs.getFloat("staked_amount", 0.0f);
            
            stakingInfo.putBoolean("hasStake", stakedAmount > 0);
            if (stakedAmount > 0) {
                long stakeTime = prefs.getLong("stake_time", 0);
                long unstakeTime = prefs.getLong("unstake_time", 0);
                double apy = prefs.getFloat("staking_apy", 0.0f);
                
                long currentTime = System.currentTimeMillis();
                double stakingDays = (currentTime - stakeTime) / (24.0 * 60 * 60 * 1000);
                double currentRewards = stakedAmount * apy / 100 * stakingDays / 365;
                
                stakingInfo.putDouble("stakedAmount", stakedAmount);
                stakingInfo.putLong("stakeTime", stakeTime);
                stakingInfo.putLong("unstakeTime", unstakeTime);
                stakingInfo.putDouble("apy", apy);
                stakingInfo.putDouble("currentRewards", currentRewards);
                stakingInfo.putBoolean("canUnstake", currentTime >= unstakeTime);
                stakingInfo.putLong("timeToUnstake", Math.max(0, unstakeTime - currentTime));
            }

            promise.resolve(stakingInfo);

        } catch (Exception e) {
            Log.e(TAG, "Failed to get staking info", e);
            promise.reject("STAKING_INFO_FAILED", e.getMessage());
        }
    }

    @ReactMethod
    public void connectEWallet(ReadableMap eWalletData, Promise promise) {
        executor.execute(() -> {
            try {
                String provider = eWalletData.getString("provider"); // ovo, gopay, dana, shopeepay
                String phoneNumber = eWalletData.getString("phoneNumber");
                String pin = eWalletData.getString("pin");

                if (!isValidEWalletProvider(provider)) {
                    promise.reject("INVALID_PROVIDER", "Unsupported e-wallet provider");
                    return;
                }

                // Mock e-wallet connection validation
                Thread.sleep(1500); // Simulate API call
                
                boolean connectionSuccess = validateEWalletCredentials(provider, phoneNumber, pin);
                if (!connectionSuccess) {
                    promise.reject("CONNECTION_FAILED", "Failed to connect to " + provider);
                    return;
                }

                // Store e-wallet connection
                SharedPreferences.Editor editor = prefs.edit();
                editor.putString("ewallet_provider", provider);
                editor.putString("ewallet_phone", phoneNumber);
                editor.putBoolean("ewallet_connected", true);
                editor.putLong("ewallet_connected_at", System.currentTimeMillis());
                editor.apply();

                WritableMap response = Arguments.createMap();
                response.putBoolean("success", true);
                response.putString("provider", provider);
                response.putString("phoneNumber", phoneNumber);
                response.putString("message", provider.toUpperCase() + " connected successfully!");

                promise.resolve(response);
                sendEvent("onEWalletConnected", response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to connect e-wallet", e);
                promise.reject("EWALLET_CONNECTION_FAILED", e.getMessage());
            }
        });
    }

    @ReactMethod
    public void withdrawToEWallet(ReadableMap withdrawData, Promise promise) {
        executor.execute(() -> {
            try {
                double amount = withdrawData.getDouble("amount");
                String tokenType = withdrawData.hasKey("tokenType") ? 
                    withdrawData.getString("tokenType") : "usdfin";

                if (!prefs.getBoolean("ewallet_connected", false)) {
                    promise.reject("NO_EWALLET", "No e-wallet connected. Connect an e-wallet first.");
                    return;
                }

                double balance = getCurrentBalance(tokenType);
                if (amount > balance) {
                    promise.reject("INSUFFICIENT_FUNDS", "Not enough " + tokenType.toUpperCase());
                    return;
                }

                // Mock withdrawal process
                String provider = prefs.getString("ewallet_provider", "");
                String phoneNumber = prefs.getString("ewallet_phone", "");
                
                Thread.sleep(3000); // Simulate processing time
                
                // Convert crypto to IDR (mock rate: 1 USDfin = 15000 IDR)
                double idrAmount = amount * 15000;
                
                // Update balance
                updateBalance(tokenType, balance - amount);

                WritableMap response = Arguments.createMap();
                response.putBoolean("success", true);
                response.putDouble("cryptoAmount", amount);
                response.putString("tokenType", tokenType);
                response.putDouble("idrAmount", idrAmount);
                response.putString("provider", provider);
                response.putString("phoneNumber", phoneNumber);
                response.putString("transactionId", "wd_" + System.currentTimeMillis());
                response.putString("message", "Withdrawal successful!");

                promise.resolve(response);
                sendEvent("onWithdrawalCompleted", response);

            } catch (Exception e) {
                Log.e(TAG, "Failed to withdraw to e-wallet", e);
                promise.reject("WITHDRAWAL_FAILED", e.getMessage());
            }
        });
    }

    // Helper methods
    private double getCurrentBalance(String tokenType) {
        return prefs.getFloat(tokenType + "_balance", 0.0f);
    }

    private void updateBalance(String tokenType, double newBalance) {
        SharedPreferences.Editor editor = prefs.edit();
        editor.putFloat(tokenType + "_balance", (float)newBalance);
        editor.apply();
    }

    private double calculateStakingAPY(double amount) {
        // Tiered APY based on staking amount
        if (amount >= 10000) return 15.0;
        if (amount >= 5000) return 12.0;
        if (amount >= 1000) return 10.0;
        return 8.0;
    }

    private boolean isValidEWalletProvider(String provider) {
        String[] validProviders = {"ovo", "gopay", "dana", "shopeepay"};
        for (String validProvider : validProviders) {
            if (validProvider.equals(provider.toLowerCase())) {
                return true;
            }
        }
        return false;
    }

    private boolean validateEWalletCredentials(String provider, String phone, String pin) {
        // Mock validation - implement actual e-wallet API validation
        return phone.length() >= 10 && pin.length() >= 4;
    }

    private void sendEvent(String eventName, WritableMap data) {
        if (reactContext.hasActiveCatalystInstance()) {
            reactContext.getJSModule(DeviceEventManagerModule.RCTDeviceEventEmitter.class)
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
