package com.finova.reactnative;

/**
 * Data classes for FinovaClientModule
 */

// Authentication result class
class AuthResult {
    private boolean success;
    private String userId;
    private String walletAddress;
    private String accessToken;
    private String refreshToken;
    private double balance;
    private int xp;
    private int rp;
    private int level;
    private String tier;
    private String errorMessage;
    private long expiresAt;
    
    public AuthResult() {
        this.success = false;
    }
    
    public AuthResult(boolean success, String userId, String walletAddress, String accessToken) {
        this.success = success;
        this.userId = userId;
        this.walletAddress = walletAddress;
        this.accessToken = accessToken;
    }
    
    // Getters and setters
    public boolean isSuccess() { return success; }
    public void setSuccess(boolean success) { this.success = success; }
    
    public String getUserId() { return userId; }
    public void setUserId(String userId) { this.userId = userId; }
    
    public String getWalletAddress() { return walletAddress; }
    public void setWalletAddress(String walletAddress) { this.walletAddress = walletAddress; }
    
    public String getAccessToken() { return accessToken; }
    public void setAccessToken(String accessToken) { this.accessToken = accessToken; }
    
    public String getRefreshToken() { return refreshToken; }
    public void setRefreshToken(String refreshToken) { this.refreshToken = refreshToken; }
    
    public double getBalance() { return balance; }
    public void setBalance(double balance) { this.balance = balance; }
    
    public int getXP() { return xp; }
    public void setXP(int xp) { this.xp = xp; }
    
    public int getRP() { return rp; }
    public void setRP(int rp) { this.rp = rp; }
    
    public int getLevel() { return level; }
    public void setLevel(int level) { this.level = level; }
    
    public String getTier() { return tier; }
    public void setTier(String tier) { this.tier = tier; }
    
    public String getErrorMessage() { return errorMessage; }
    public void setErrorMessage(String errorMessage) { this.errorMessage = errorMessage; }
    
    public long getExpiresAt() { return expiresAt; }
    public void setExpiresAt(long expiresAt) { this.expiresAt = expiresAt; }
}

// User profile class
class UserProfile {
    private String userId;
    private String email;
    private String username;
    private String bio;
    private String avatar;
    private String walletAddress;
    private double balance;
    private int xp;
    private int rp;
    private int level;
    private String tier;
    private double miningRate;
    private int referralCount;
    private boolean isKYCVerified;
    private boolean isActive;
    private String createdAt;
    private String lastActiveAt;
    
    // Stats
    private double totalMined;
    private int totalXPEarned;
    private int totalRPEarned;
    private int streakDays;
    private int postsCount;
    private int commentsCount;
    private int likesReceived;
    private int sharesReceived;
    
    // Social connections
    private int followersCount;
    private int followingCount;
    private int friendsCount;
    
    // Guild info
    private String guildId;
    private String guildName;
    private String guildRole;
    
    // NFTs and special cards
    private int nftCount;
    private int specialCardsCount;
    
    public UserProfile() {}
    
    // Getters and setters
    public String getUserId() { return userId; }
    public void setUserId(String userId) { this.userId = userId; }
    
    public String getEmail() { return email; }
    public void setEmail(String email) { this.email = email; }
    
    public String getUsername() { return username; }
    public void setUsername(String username) { this.username = username; }
    
    public String getBio() { return bio; }
    public void setBio(String bio) { this.bio = bio; }
    
    public String getAvatar() { return avatar; }
    public void setAvatar(String avatar) { this.avatar = avatar; }
    
    public String getWalletAddress() { return walletAddress; }
    public void setWalletAddress(String walletAddress) { this.walletAddress = walletAddress; }
    
    public double getBalance() { return balance; }
    public void setBalance(double balance) { this.balance = balance; }
    
    public int getXP() { return xp; }
    public void setXP(int xp) { this.xp = xp; }
    
    public int getRP() { return rp; }
    public void setRP(int rp) { this.rp = rp; }
    
    public int getLevel() { return level; }
    public void setLevel(int level) { this.level = level; }
    
    public String getTier() { return tier; }
    public void setTier(String tier) { this.tier = tier; }
    
    public double getMiningRate() { return miningRate; }
    public void setMiningRate(double miningRate) { this.miningRate = miningRate; }
    
    public int getReferralCount() { return referralCount; }
    public void setReferralCount(int referralCount) { this.referralCount = referralCount; }
    
    public boolean isKYCVerified() { return isKYCVerified; }
    public void setKYCVerified(boolean isKYCVerified) { this.isKYCVerified = isKYCVerified; }
    
    public boolean isActive() { return isActive; }
    public void setActive(boolean isActive) { this.isActive = isActive; }
    
    public String getCreatedAt() { return createdAt; }
    public void setCreatedAt(String createdAt) { this.createdAt = createdAt; }
    
    public String getLastActiveAt() { return lastActiveAt; }
    public void setLastActiveAt(String lastActiveAt) { this.lastActiveAt = lastActiveAt; }
    
    // Stats getters/setters
    public double getTotalMined() { return totalMined; }
    public void setTotalMined(double totalMined) { this.totalMined = totalMined; }
    
    public int getTotalXPEarned() { return totalXPEarned; }
    public void setTotalXPEarned(int totalXPEarned) { this.totalXPEarned = totalXPEarned; }
    
    public int getTotalRPEarned() { return totalRPEarned; }
    public void setTotalRPEarned(int totalRPEarned) { this.totalRPEarned = totalRPEarned; }
    
    public int getStreakDays() { return streakDays; }
    public void setStreakDays(int streakDays) { this.streakDays = streakDays; }
    
    public int getPostsCount() { return postsCount; }
    public void setPostsCount(int postsCount) { this.postsCount = postsCount; }
    
    public int getCommentsCount() { return commentsCount; }
    public void setCommentsCount(int commentsCount) { this.commentsCount = commentsCount; }
    
    public int getLikesReceived() { return likesReceived; }
    public void setLikesReceived(int likesReceived) { this.likesReceived = likesReceived; }
    
    public int getSharesReceived() { return sharesReceived; }
    public void setSharesReceived(int sharesReceived) { this.sharesReceived = sharesReceived; }
    
    public int getFollowersCount() { return followersCount; }
    public void setFollowersCount(int followersCount) { this.followersCount = followersCount; }
    
    public int getFollowingCount() { return followingCount; }
    public void setFollowingCount(int followingCount) { this.followingCount = followingCount; }
    
    public int getFriendsCount() { return friendsCount; }
    public void setFriendsCount(int friendsCount) { this.friendsCount = friendsCount; }
    
    public String getGuildId() { return guildId; }
    public void setGuildId(String guildId) { this.guildId = guildId; }
    
    public String getGuildName() { return guildName; }
    public void setGuildName(String guildName) { this.guildName = guildName; }
    
    public String getGuildRole() { return guildRole; }
    public void setGuildRole(String guildRole) { this.guildRole = guildRole; }
    
    public int getNftCount() { return nftCount; }
    public void setNftCount(int nftCount) { this.nftCount = nftCount; }
    
    public int getSpecialCardsCount() { return specialCardsCount; }
    public void setSpecialCardsCount(int specialCardsCount) { this.specialCardsCount = specialCardsCount; }
}

// User profile update class
class UserProfileUpdate {
    private String username;
    private String email;
    private String bio;
    private String avatar;
    private String firstName;
    private String lastName;
    private String phoneNumber;
    private String country;
    private String language;
    private String timezone;
    
    // Privacy settings
    private boolean isProfilePublic;
    private boolean allowDirectMessages;
    private boolean showActivity;
    private boolean allowReferrals;
    
    // Notification preferences
    private boolean emailNotifications;
    private boolean pushNotifications;
    private boolean smsNotifications;
    private boolean miningAlerts;
    private boolean socialAlerts;
    
    public UserProfileUpdate() {}
    
    // Getters and setters
    public String getUsername() { return username; }
    public void setUsername(String username) { this.username = username; }
    
    public String getEmail() { return email; }
    public void setEmail(String email) { this.email = email; }
    
    public String getBio() { return bio; }
    public void setBio(String bio) { this.bio = bio; }
    
    public String getAvatar() { return avatar; }
    public void setAvatar(String avatar) { this.avatar = avatar; }
    
    public String getFirstName() { return firstName; }
    public void setFirstName(String firstName) { this.firstName = firstName; }
    
    public String getLastName() { return lastName; }
    public void setLastName(String lastName) { this.lastName = lastName; }
    
    public String getPhoneNumber() { return phoneNumber; }
    public void setPhoneNumber(String phoneNumber) { this.phoneNumber = phoneNumber; }
    
    public String getCountry() { return country; }
    public void setCountry(String country) { this.country = country; }
    
    public String getLanguage() { return language; }
    public void setLanguage(String language) { this.language = language; }
    
    public String getTimezone() { return timezone; }
    public void setTimezone(String timezone) { this.timezone = timezone; }
    
    public boolean isProfilePublic() { return isProfilePublic; }
    public void setProfilePublic(boolean isProfilePublic) { this.isProfilePublic = isProfilePublic; }
    
    public boolean isAllowDirectMessages() { return allowDirectMessages; }
    public void setAllowDirectMessages(boolean allowDirectMessages) { this.allowDirectMessages = allowDirectMessages; }
    
    public boolean isShowActivity() { return showActivity; }
    public void setShowActivity(boolean showActivity) { this.showActivity = showActivity; }
    
    public boolean isAllowReferrals() { return allowReferrals; }
    public void setAllowReferrals(boolean allowReferrals) { this.allowReferrals = allowReferrals; }
    
    public boolean isEmailNotifications() { return emailNotifications; }
    public void setEmailNotifications(boolean emailNotifications) { this.emailNotifications = emailNotifications; }
    
    public boolean isPushNotifications() { return pushNotifications; }
    public void setPushNotifications(boolean pushNotifications) { this.pushNotifications = pushNotifications; }
    
    public boolean isSmsNotifications() { return smsNotifications; }
    public void setSmsNotifications(boolean smsNotifications) { this.smsNotifications = smsNotifications; }
    
    public boolean isMiningAlerts() { return miningAlerts; }
    public void setMiningAlerts(boolean miningAlerts) { this.miningAlerts = miningAlerts; }
    
    public boolean isSocialAlerts() { return socialAlerts; }
    public void setSocialAlerts(boolean socialAlerts) { this.socialAlerts = socialAlerts; }
}
