package com.finova.reactnative;

/**
 * Data classes for wallet operations
 */

// Wallet information class
class WalletInfo {
    private String type;
    private String address;
    private String publicKey;
    private String network;
    private boolean isConnected;
    private double balance;
    private String name;
    private String icon;
    private long lastUpdated;
    
    public WalletInfo() {
        this.lastUpdated = System.currentTimeMillis();
    }
    
    // Getters and setters
    public String getType() { return type; }
    public void setType(String type) { this.type = type; }
    
    public String getAddress() { return address; }
    public void setAddress(String address) { this.address = address; }
    
    public String getPublicKey() { return publicKey; }
    public void setPublicKey(String publicKey) { this.publicKey = publicKey; }
    
    public String getNetwork() { return network; }
    public void setNetwork(String network) { this.network = network; }
    
    public boolean isConnected() { return isConnected; }
    public void setConnected(boolean connected) { this.isConnected = connected; }
    
    public double getBalance() { return balance; }
    public void setBalance(double balance) { this.balance = balance; }
    
    public String getName() { return name; }
    public void setName(String name) { this.name = name; }
    
    public String getIcon() { return icon; }
    public void setIcon(String icon) { this.icon = icon; }
    
    public long getLastUpdated() { return lastUpdated; }
    public void setLastUpdated(long lastUpdated) { this.lastUpdated = lastUpdated; }
}

// Wallet connection result class
class WalletConnectionResult {
    private boolean success;
    private String address;
    private String publicKey;
    private String network;
    private double balance;
    private String errorMessage;
    private String sessionToken;
    
    public WalletConnectionResult() {}
    
    public WalletConnectionResult(boolean success, String address, String publicKey, String network, double balance) {
        this.success = success;
        this.address = address;
        this.publicKey = publicKey;
        this.network = network;
        this.balance = balance;
    }
    
    // Getters and setters
    public boolean isSuccess() { return success; }
    public void setSuccess(boolean success) { this.success = success; }
    
    public String getAddress() { return address; }
    public void setAddress(String address) { this.address = address; }
    
    public String getPublicKey() { return publicKey; }
    public void setPublicKey(String publicKey) { this.publicKey = publicKey; }
    
    public String getNetwork() { return network; }
    public void setNetwork(String network) { this.network = network; }
    
    public double getBalance() { return balance; }
    public void setBalance(double balance) { this.balance = balance; }
    
    public String getErrorMessage() { return errorMessage; }
    public void setErrorMessage(String errorMessage) { this.errorMessage = errorMessage; }
    
    public String getSessionToken() { return sessionToken; }
    public void setSessionToken(String sessionToken) { this.sessionToken = sessionToken; }
}
