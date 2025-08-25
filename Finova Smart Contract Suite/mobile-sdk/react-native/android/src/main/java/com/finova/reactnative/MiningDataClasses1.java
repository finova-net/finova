package com.finova.reactnative;

import java.util.List;
import java.util.ArrayList;
import java.util.UUID;

/**
 * Data classes for mining operations
 */

// Mining session class
class MiningSession {
    private String sessionId;
    private String userId;
    private long startTime;
    private long endTime;
    private long duration;
    private double miningRate;
    private double baseRate;
    private double finizenBonus;
    private double referralBonus;
    private double securityBonus;
    private double regressionFactor;
    private boolean isActive;
    private double minedAmount;
    private List<MiningBoost> activeBoosts;
    
    public MiningSession() {
        this.sessionId = UUID.randomUUID().toString();
        this.activeBoosts = new ArrayList<>();
        this.isActive = false;
        this.minedAmount = 0.0;
    }
    
    public long getRemainingTime() {
        if (!isActive) return 0;
        long elapsed = System.currentTimeMillis() - startTime;
        return Math.max(0, duration - elapsed);
    }
    
    public void applyBoost(MiningBoost boost) {
        activeBoosts.add(boost);
        // Recalculate mining rate with boosts
        double totalMultiplier = 1.0;
        for (MiningBoost b : activeBoosts) {
            if (b.isActive()) {
                totalMultiplier *= b.getMultiplier();
            }
        }
        this.miningRate = this.baseRate * this.finizenBonus * this.referralBonus * 
                         this.securityBonus * this.regressionFactor * totalMultiplier;
    }
    
    // Getters and setters
    public String getSessionId() { return sessionId; }
    public void setSessionId(String sessionId) { this.sessionId = sessionId; }
    
    public String getUserId() { return userId; }
    public void setUserId(String userId) { this.userId = userId; }
    
    public long getStartTime() { return startTime; }
    public void setStartTime(long startTime) { this.startTime = startTime; }
    
    public long getEndTime() { return endTime; }
    public void setEndTime(long endTime) { this.endTime = endTime; }
    
    public long getDuration() { return duration; }
    public void setDuration(long duration) { this.duration = duration; }
    
    public double getMiningRate() { return miningRate; }
    public void setMiningRate(double miningRate) { this.miningRate = miningRate; }
    
    public double getBaseRate() { return baseRate; }
    public void setBaseRate(double baseRate) { this.baseRate = baseRate; }
    
    public double getFinizenBonus() { return finizenBonus; }
    public void setFinizenBonus(double finizenBonus) { this.finizenBonus = finizenBonus; }
    
    public double getReferralBonus() { return referralBonus; }
    public void setReferralBonus(double referralBonus) { this.referralBonus = referralBonus; }
    
    public double getSecurityBonus() { return securityBonus; }
    public void setSecurityBonus(double securityBonus) { this.securityBonus = securityBonus; }
    
    public double getRegressionFactor() { return regressionFactor; }
    public void setRegressionFactor(double regressionFactor) { this.regressionFactor = regressionFactor; }
    
    public boolean isActive() { return isActive; }
    public void setActive(boolean active) { this.isActive = active; }
    
    public double getMinedAmount() { return minedAmount; }
    public void setMinedAmount(double minedAmount) { this.minedAmount = minedAmount; }
    
    public List<MiningBoost> getActiveBoosts() { return activeBoosts; }
    public void setActiveBoosts(List<MiningBoost> activeBoosts) { this.activeBoosts = activeBoosts; }
}

// User mining data class
class UserMiningData {
    private String userId;
    private int totalNetworkUsers;
    private int activeReferrals;
    private double totalHoldings;
    private boolean isKYCVerified;
    private int xpLevel;
    private String rpTier;
    private int streakDays;
    private double lastMiningRate;
    private long lastMiningTime;
    private boolean hasActiveSession;
    
    public UserMiningData() {
        this.totalNetworkUsers = 50000;
        this.activeReferrals = 0;
        this.totalHoldings = 0.0;
        this.isKYCVerified = false;
        this.xpLevel = 1;
        this.rpTier = "Explorer";
        this.streakDays = 0;
        this.hasActiveSession = false;
    }
    
    // Getters and setters
    public String getUserId() { return userId; }
    public void setUserId(String userId) { this.userId = userId; }
    
    public int getTotalNetworkUsers() { return totalNetworkUsers; }
    public void setTotalNetworkUsers(int totalNetworkUsers) { this.totalNetworkUsers = totalNetworkUsers; }
    
    public int getActiveReferrals() { return activeReferrals; }
    public void setActiveReferrals(int activeReferrals) { this.activeReferrals = activeReferrals; }
    
    public double getTotalHoldings() { return totalHoldings; }
    public void setTotalHoldings(double totalHoldings) { this.totalHoldings = totalHoldings; }
    
    public boolean isKYCVerified() { return isKYCVerified; }
    public void setKYCVerified(boolean isKYCVerified) { this.isKYCVerified = isKYCVerified; }
    
    public int getXpLevel() { return xpLevel; }
    public void setXpLevel(int xpLevel) { this.xpLevel = xpLevel; }
    
    public String getRpTier() { return rpTier; }
    public void setRpTier(String rpTier) { this.rpTier = rpTier; }
    
    public int getStreakDays() { return streakDays; }
    public void setStreakDays(int streakDays) { this.streakDays = streakDays; }
    
    public double getLastMiningRate() { return lastMiningRate; }
    public void setLastMiningRate(double lastMiningRate) { this.lastMiningRate = lastMiningRate; }
    
    public long getLastMiningTime() { return lastMiningTime; }
    public void setLastMiningTime(long lastMiningTime) { this.lastMiningTime = lastMiningTime; }
    
    public boolean hasActiveSession() { return hasActiveSession; }
    public void setHasActiveSession(boolean hasActiveSession) { this.hasActiveSession = hasActiveSession; }
}

// Mining rate calculation class
class MiningRateCalculation {
    private double finalRate;
    private double baseRate;
    private double finizenBonus;
    private double referralBonus;
    private double securityBonus;
    private double regressionFactor;
    private String miningPhase;
    private double xpMultiplier;
    private double rpMultiplier;
    
    public MiningRateCalculation() {}
    
    // Getters and setters
    public double getFinalRate() { return finalRate; }
    public void setFinalRate(double finalRate) { this.finalRate = finalRate; }
    
    public double getBaseRate() { return baseRate; }
    public void setBaseRate(double baseRate) { this.baseRate = baseRate; }
    
    public double getFinizenBonus() { return finizenBonus; }
    public void setFinizenBonus(double finizenBonus) { this.finizenBonus = finizenBonus; }
    
    public double getReferralBonus() { return referralBonus; }
    public void setReferralBonus(double referralBonus) { this.referralBonus = referralBonus; }
    
    public double getSecurityBonus() { return securityBonus; }
    public void setSecurityBonus(double securityBonus) { this.securityBonus = securityBonus; }
    
    public double getRegressionFactor() { return regressionFactor; }
    public void setRegressionFactor(double regressionFactor) { this.regressionFactor = regressionFactor; }
    
    public String getMiningPhase() { return miningPhase; }
    public void setMiningPhase(String miningPhase) { this.miningPhase = miningPhase; }
    
    public double getXpMultiplier() { return xpMultiplier; }
    public void setXpMultiplier(double xpMultiplier) { this.xpMultiplier = xpMultiplier; }
    
    public double getRpMultiplier() { return rpMultiplier; }
    public void setRpMultiplier(double rpMultiplier) { this.rpMultiplier = rpMultiplier; }
}

// Mining boost class
class MiningBoost {
    private String boostId;
    private double multiplier;
    private long duration;
    private long startTime;
    private String cardType;
    private boolean isActive;
    
    public MiningBoost(double multiplier, long duration) {
        this.boostId = UUID.randomUUID().toString();
        this.multiplier = multiplier;
        this.duration = duration;
        this.startTime = System.currentTimeMillis();
        this.isActive = true;
    }
    
    public boolean isActive() {
        if (!isActive) return false;
        long elapsed = System.currentTimeMillis() - startTime;
        if (elapsed >= duration) {
            isActive = false;
            return false;
        }
        return true;
    }
    
    public long getRemainingTime() {
        if (!isActive) return 0;
        long elapsed = System.currentTimeMillis() - startTime;
        return Math.max(0, duration - elapsed);
    }
    
    // Getters and setters
    public String getBoostId() { return boostId; }
    public void setBoostId(String boostId) { this.boostId = boostId; }
    
    public double getMultiplier() { return multiplier; }
    public void setMultiplier(double multiplier) { this.multiplier = multiplier; }
    
    public long getDuration() { return duration; }
    public void setDuration(long duration) { this.duration = duration; }
    
    public long getStartTime() { return startTime; }
    public void setStartTime(long startTime) { this.startTime = startTime; }
    
    public String getCardType() { return cardType; }
    public void setCardType(String cardType) { this.cardType = cardType; }
    
    public void setActive(boolean active) { this.isActive = active; }
}
