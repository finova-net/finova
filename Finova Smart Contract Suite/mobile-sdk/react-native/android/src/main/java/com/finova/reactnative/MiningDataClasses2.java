package com.finova.reactnative;

import java.util.List;
import java.util.ArrayList;

/**
 * Additional data classes for mining operations
 */

// User mining statistics class
class UserMiningStats {
    private String userId;
    private double totalMined;
    private int totalSessions;
    private int completedSessions;
    private double averageRate;
    private long totalMiningTime;
    private int streakDays;
    private long lastMiningTime;
    private List<DailyMiningStats> dailyStats;
    private List<MiningMilestone> milestones;
    private int rank;
    private double efficiency;
    
    public UserMiningStats() {
        this.dailyStats = new ArrayList<>();
        this.milestones = new ArrayList<>();
        this.totalMined = 0.0;
        this.totalSessions = 0;
        this.completedSessions = 0;
        this.averageRate = 0.0;
        this.totalMiningTime = 0;
        this.streakDays = 0;
        this.rank = 0;
        this.efficiency = 0.0;
    }
    
    // Getters and setters
    public String getUserId() { return userId; }
    public void setUserId(String userId) { this.userId = userId; }
    
    public double getTotalMined() { return totalMined; }
    public void setTotalMined(double totalMined) { this.totalMined = totalMined; }
    
    public int getTotalSessions() { return totalSessions; }
    public void setTotalSessions(int totalSessions) { this.totalSessions = totalSessions; }
    
    public int getCompletedSessions() { return completedSessions; }
    public void setCompletedSessions(int completedSessions) { this.completedSessions = completedSessions; }
    
    public double getAverageRate() { return averageRate; }
    public void setAverageRate(double averageRate) { this.averageRate = averageRate; }
    
    public long getTotalMiningTime() { return totalMiningTime; }
    public void setTotalMiningTime(long totalMiningTime) { this.totalMiningTime = totalMiningTime; }
    
    public int getStreakDays() { return streakDays; }
    public void setStreakDays(int streakDays) { this.streakDays = streakDays; }
    
    public long getLastMiningTime() { return lastMiningTime; }
    public void setLastMiningTime(long lastMiningTime) { this.lastMiningTime = lastMiningTime; }
    
    public List<DailyMiningStats> getDailyStats() { return dailyStats; }
    public void setDailyStats(List<DailyMiningStats> dailyStats) { this.dailyStats = dailyStats; }
    
    public List<MiningMilestone> getMilestones() { return milestones; }
    public void setMilestones(List<MiningMilestone> milestones) { this.milestones = milestones; }
    
    public int getRank() { return rank; }
    public void setRank(int rank) { this.rank = rank; }
    
    public double getEfficiency() { return efficiency; }
    public void setEfficiency(double efficiency) { this.efficiency = efficiency; }
}

// Daily mining statistics class
class DailyMiningStats {
    private String date;
    private double amountMined;
    private int sessionsCount;
    private double averageRate;
    private long totalTime;
    private boolean streakMaintained;
    
    public DailyMiningStats() {}
    
    public DailyMiningStats(String date, double amountMined, int sessionsCount) {
        this.date = date;
        this.amountMined = amountMined;
        this.sessionsCount = sessionsCount;
    }
    
    // Getters and setters
    public String getDate() { return date; }
    public void setDate(String date) { this.date = date; }
    
    public double getAmountMined() { return amountMined; }
    public void setAmountMined(double amountMined) { this.amountMined = amountMined; }
    
    public int getSessionsCount() { return sessionsCount; }
    public void setSessionsCount(int sessionsCount) { this.sessionsCount = sessionsCount; }
    
    public double getAverageRate() { return averageRate; }
    public void setAverageRate(double averageRate) { this.averageRate = averageRate; }
    
    public long getTotalTime() { return totalTime; }
    public void setTotalTime(long totalTime) { this.totalTime = totalTime; }
    
    public boolean isStreakMaintained() { return streakMaintained; }
    public void setStreakMaintained(boolean streakMaintained) { this.streakMaintained = streakMaintained; }
}

// Mining milestone class
class MiningMilestone {
    private String id;
    private String name;
    private String description;
    private double target;
    private double current;
    private boolean achieved;
    private long achievedAt;
    private double reward;
    private String category;
    private int priority;
    
    public MiningMilestone() {}
    
    public MiningMilestone(String name, String description, double target) {
        this.name = name;
        this.description = description;
        this.target = target;
        this.current = 0.0;
        this.achieved = false;
        this.achievedAt = 0;
    }
    
    public double getProgress() {
        return target > 0 ? (current / target) * 100.0 : 0.0;
    }
    
    // Getters and setters
    public String getId() { return id; }
    public void setId(String id) { this.id = id; }
    
    public String getName() { return name; }
    public void setName(String name) { this.name = name; }
    
    public String getDescription() { return description; }
    public void setDescription(String description) { this.description = description; }
    
    public double getTarget() { return target; }
    public void setTarget(double target) { this.target = target; }
    
    public double getCurrent() { return current; }
    public void setCurrent(double current) { this.current = current; }
    
    public boolean isAchieved() { return achieved; }
    public void setAchieved(boolean achieved) { this.achieved = achieved; }
    
    public long getAchievedAt() { return achievedAt; }
    public void setAchievedAt(long achievedAt) { this.achievedAt = achievedAt; }
    
    public double getReward() { return reward; }
    public void setReward(double reward) { this.reward = reward; }
    
    public String getCategory() { return category; }
    public void setCategory(String category) { this.category = category; }
    
    public int getPriority() { return priority; }
    public void setPriority(int priority) { this.priority = priority; }
}

// Global mining information class
class GlobalMiningInfo {
    private int totalUsers;
    private int activeMiners;
    private double totalMined;
    private double currentBaseRate;
    private String currentPhase;
    private int usersToNextPhase;
    private double networkHashRate;
    private List<MiningPhaseInfo> phases;
    private long lastUpdated;
    
    public GlobalMiningInfo() {
        this.phases = new ArrayList<>();
        this.lastUpdated = System.currentTimeMillis();
        this.totalUsers = 50000;
        this.activeMiners = 25000;
        this.totalMined = 1000000.0;
        this.currentBaseRate = 0.05;
        this.currentPhase = "growth";
        this.usersToNextPhase = 450000;
        this.networkHashRate = 1000.0;
    }
    
    // Getters and setters
    public int getTotalUsers() { return totalUsers; }
    public void setTotalUsers(int totalUsers) { this.totalUsers = totalUsers; }
    
    public int getActiveMiners() { return activeMiners; }
    public void setActiveMiners(int activeMiners) { this.activeMiners = activeMiners; }
    
    public double getTotalMined() { return totalMined; }
    public void setTotalMined(double totalMined) { this.totalMined = totalMined; }
    
    public double getCurrentBaseRate() { return currentBaseRate; }
    public void setCurrentBaseRate(double currentBaseRate) { this.currentBaseRate = currentBaseRate; }
    
    public String getCurrentPhase() { return currentPhase; }
    public void setCurrentPhase(String currentPhase) { this.currentPhase = currentPhase; }
    
    public int getUsersToNextPhase() { return usersToNextPhase; }
    public void setUsersToNextPhase(int usersToNextPhase) { this.usersToNextPhase = usersToNextPhase; }
    
    public double getNetworkHashRate() { return networkHashRate; }
    public void setNetworkHashRate(double networkHashRate) { this.networkHashRate = networkHashRate; }
    
    public List<MiningPhaseInfo> getPhases() { return phases; }
    public void setPhases(List<MiningPhaseInfo> phases) { this.phases = phases; }
    
    public long getLastUpdated() { return lastUpdated; }
    public void setLastUpdated(long lastUpdated) { this.lastUpdated = lastUpdated; }
}

// Mining phase information class
class MiningPhaseInfo {
    private String name;
    private String description;
    private int userRange;
    private double baseRate;
    private double finizenBonus;
    private boolean active;
    private long startTime;
    private long estimatedEndTime;
    
    public MiningPhaseInfo() {}
    
    public MiningPhaseInfo(String name, int userRange, double baseRate, boolean active) {
        this.name = name;
        this.userRange = userRange;
        this.baseRate = baseRate;
        this.active = active;
    }
    
    // Getters and setters
    public String getName() { return name; }
    public void setName(String name) { this.name = name; }
    
    public String getDescription() { return description; }
    public void setDescription(String description) { this.description = description; }
    
    public int getUserRange() { return userRange; }
    public void setUserRange(int userRange) { this.userRange = userRange; }
    
    public double getBaseRate() { return baseRate; }
    public void setBaseRate(double baseRate) { this.baseRate = baseRate; }
    
    public double getFinizenBonus() { return finizenBonus; }
    public void setFinizenBonus(double finizenBonus) { this.finizenBonus = finizenBonus; }
    
    public boolean isActive() { return active; }
    public void setActive(boolean active) { this.active = active; }
    
    public long getStartTime() { return startTime; }
    public void setStartTime(long startTime) { this.startTime = startTime; }
    
    public long getEstimatedEndTime() { return estimatedEndTime; }
    public void setEstimatedEndTime(long estimatedEndTime) { this.estimatedEndTime = estimatedEndTime; }
}
