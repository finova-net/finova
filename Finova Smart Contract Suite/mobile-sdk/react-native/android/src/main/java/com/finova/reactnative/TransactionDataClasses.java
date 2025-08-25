package com.finova.reactnative;

import java.util.Map;
import java.util.HashMap;
import java.util.List;
import java.util.ArrayList;

/**
 * Data classes for transaction operations
 */

// Transaction builder class
class TransactionBuilder {
    private String fromAddress;
    private String toAddress;
    private double amount;
    private String type;
    private String memo;
    private String tokenMint;
    private Map<String, Object> instructions;
    private int priorityFee = 1000;
    
    public TransactionBuilder() {
        this.instructions = new HashMap<>();
    }
    
    public TransactionBuilder setFromAddress(String fromAddress) {
        this.fromAddress = fromAddress;
        return this;
    }
    
    public TransactionBuilder setToAddress(String toAddress) {
        this.toAddress = toAddress;
        return this;
    }
    
    public TransactionBuilder setAmount(double amount) {
        this.amount = amount;
        return this;
    }
    
    public TransactionBuilder setType(String type) {
        this.type = type;
        return this;
    }
    
    public TransactionBuilder setMemo(String memo) {
        this.memo = memo;
        return this;
    }
    
    public TransactionBuilder setTokenMint(String tokenMint) {
        this.tokenMint = tokenMint;
        return this;
    }
    
    public TransactionBuilder addInstruction(String name, Map<String, Object> params) {
        this.instructions.put(name, params);
        return this;
    }
    
    public TransactionBuilder setPriorityFee(int priorityFee) {
        this.priorityFee = priorityFee;
        return this;
    }
    
    // Getters
    public String getFromAddress() { return fromAddress; }
    public String getToAddress() { return toAddress; }
    public double getAmount() { return amount; }
    public String getType() { return type; }
    public String getMemo() { return memo; }
    public String getTokenMint() { return tokenMint; }
    public Map<String, Object> getInstructions() { return instructions; }
    public int getPriorityFee() { return priorityFee; }
}

// Transaction result class
class TransactionResult {
    private boolean success;
    private String transactionId;
    private String errorMessage;
    private String signature;
    private long timestamp;
    
    public TransactionResult() {}
    
    public TransactionResult(boolean success, String transactionId) {
        this.success = success;
        this.transactionId = transactionId;
        this.timestamp = System.currentTimeMillis();
    }
    
    // Getters and setters
    public boolean isSuccess() { return success; }
    public void setSuccess(boolean success) { this.success = success; }
    
    public String getTransactionId() { return transactionId; }
    public void setTransactionId(String transactionId) { this.transactionId = transactionId; }
    
    public String getErrorMessage() { return errorMessage; }
    public void setErrorMessage(String errorMessage) { this.errorMessage = errorMessage; }
    
    public String getSignature() { return signature; }
    public void setSignature(String signature) { this.signature = signature; }
    
    public long getTimestamp() { return timestamp; }
    public void setTimestamp(long timestamp) { this.timestamp = timestamp; }
}

// Batch transaction result class
class BatchTransactionResult {
    private boolean success;
    private List<String> transactionIds;
    private String errorMessage;
    private int successCount;
    private int failureCount;
    
    public BatchTransactionResult() {
        this.transactionIds = new ArrayList<>();
    }
    
    public BatchTransactionResult(boolean success, List<String> transactionIds) {
        this.success = success;
        this.transactionIds = transactionIds != null ? transactionIds : new ArrayList<>();
        this.successCount = this.transactionIds.size();
    }
    
    // Getters and setters
    public boolean isSuccess() { return success; }
    public void setSuccess(boolean success) { this.success = success; }
    
    public List<String> getTransactionIds() { return transactionIds; }
    public void setTransactionIds(List<String> transactionIds) { this.transactionIds = transactionIds; }
    
    public String getErrorMessage() { return errorMessage; }
    public void setErrorMessage(String errorMessage) { this.errorMessage = errorMessage; }
    
    public int getSuccessCount() { return successCount; }
    public void setSuccessCount(int successCount) { this.successCount = successCount; }
    
    public int getFailureCount() { return failureCount; }
    public void setFailureCount(int failureCount) { this.failureCount = failureCount; }
}

// Transaction info class
class TransactionInfo {
    private String transactionId;
    private String type;
    private String fromAddress;
    private String toAddress;
    private double amount;
    private String status;
    private String tokenSymbol;
    private String memo;
    private long timestamp;
    private double fee;
    private int confirmations;
    private String blockHash;
    
    public TransactionInfo() {}
    
    // Getters and setters
    public String getTransactionId() { return transactionId; }
    public void setTransactionId(String transactionId) { this.transactionId = transactionId; }
    
    public String getType() { return type; }
    public void setType(String type) { this.type = type; }
    
    public String getFromAddress() { return fromAddress; }
    public void setFromAddress(String fromAddress) { this.fromAddress = fromAddress; }
    
    public String getToAddress() { return toAddress; }
    public void setToAddress(String toAddress) { this.toAddress = toAddress; }
    
    public double getAmount() { return amount; }
    public void setAmount(double amount) { this.amount = amount; }
    
    public String getStatus() { return status; }
    public void setStatus(String status) { this.status = status; }
    
    public String getTokenSymbol() { return tokenSymbol; }
    public void setTokenSymbol(String tokenSymbol) { this.tokenSymbol = tokenSymbol; }
    
    public String getMemo() { return memo; }
    public void setMemo(String memo) { this.memo = memo; }
    
    public long getTimestamp() { return timestamp; }
    public void setTimestamp(long timestamp) { this.timestamp = timestamp; }
    
    public double getFee() { return fee; }
    public void setFee(double fee) { this.fee = fee; }
    
    public int getConfirmations() { return confirmations; }
    public void setConfirmations(int confirmations) { this.confirmations = confirmations; }
    
    public String getBlockHash() { return blockHash; }
    public void setBlockHash(String blockHash) { this.blockHash = blockHash; }
}

// Transaction status class
class TransactionStatus {
    private String status;
    private int confirmations;
    private String blockHash;
    private long blockTime;
    private double fee;
    private String errorMessage;
    private boolean hasError;
    
    public TransactionStatus() {}
    
    public TransactionStatus(String status, int confirmations, String blockHash, long blockTime, double fee) {
        this.status = status;
        this.confirmations = confirmations;
        this.blockHash = blockHash;
        this.blockTime = blockTime;
        this.fee = fee;
        this.hasError = false;
    }
    
    // Getters and setters
    public String getStatus() { return status; }
    public void setStatus(String status) { this.status = status; }
    
    public int getConfirmations() { return confirmations; }
    public void setConfirmations(int confirmations) { this.confirmations = confirmations; }
    
    public String getBlockHash() { return blockHash; }
    public void setBlockHash(String blockHash) { this.blockHash = blockHash; }
    
    public long getBlockTime() { return blockTime; }
    public void setBlockTime(long blockTime) { this.blockTime = blockTime; }
    
    public double getFee() { return fee; }
    public void setFee(double fee) { this.fee = fee; }
    
    public String getErrorMessage() { return errorMessage; }
    public void setErrorMessage(String errorMessage) { 
        this.errorMessage = errorMessage;
        this.hasError = errorMessage != null && !errorMessage.isEmpty();
    }
    
    public boolean hasError() { return hasError; }
    public void setHasError(boolean hasError) { this.hasError = hasError; }
}

// Transaction history class
class TransactionHistory {
    private String transactionId;
    private String type;
    private String status;
    private double amount;
    private String fromAddress;
    private String toAddress;
    private long timestamp;
    private double fee;
    private String tokenSymbol;
    private String memo;
    private int confirmations;
    private String blockHash;
    private boolean isIncoming;
    private boolean isOutgoing;
    
    public TransactionHistory() {}
    
    // Getters and setters
    public String getTransactionId() { return transactionId; }
    public void setTransactionId(String transactionId) { this.transactionId = transactionId; }
    
    public String getType() { return type; }
    public void setType(String type) { this.type = type; }
    
    public String getStatus() { return status; }
    public void setStatus(String status) { this.status = status; }
    
    public double getAmount() { return amount; }
    public void setAmount(double amount) { this.amount = amount; }
    
    public String getFromAddress() { return fromAddress; }
    public void setFromAddress(String fromAddress) { this.fromAddress = fromAddress; }
    
    public String getToAddress() { return toAddress; }
    public void setToAddress(String toAddress) { this.toAddress = toAddress; }
    
    public long getTimestamp() { return timestamp; }
    public void setTimestamp(long timestamp) { this.timestamp = timestamp; }
    
    public double getFee() { return fee; }
    public void setFee(double fee) { this.fee = fee; }
    
    public String getTokenSymbol() { return tokenSymbol; }
    public void setTokenSymbol(String tokenSymbol) { this.tokenSymbol = tokenSymbol; }
    
    public String getMemo() { return memo; }
    public void setMemo(String memo) { this.memo = memo; }
    
    public int getConfirmations() { return confirmations; }
    public void setConfirmations(int confirmations) { this.confirmations = confirmations; }
    
    public String getBlockHash() { return blockHash; }
    public void setBlockHash(String blockHash) { this.blockHash = blockHash; }
    
    public boolean isIncoming() { return isIncoming; }
    public void setIncoming(boolean incoming) { this.isIncoming = incoming; }
    
    public boolean isOutgoing() { return isOutgoing; }
    public void setOutgoing(boolean outgoing) { this.isOutgoing = outgoing; }
}

// Fee estimate class
class FeeEstimate {
    private double baseFee;
    private double priorityFee;
    private double totalFee;
    private String currency;
    private long estimatedAt;
    
    public FeeEstimate() {
        this.estimatedAt = System.currentTimeMillis();
    }
    
    public FeeEstimate(double baseFee, double priorityFee, double totalFee) {
        this.baseFee = baseFee;
        this.priorityFee = priorityFee;
        this.totalFee = totalFee;
        this.currency = "SOL";
        this.estimatedAt = System.currentTimeMillis();
    }
    
    // Getters and setters
    public double getBaseFee() { return baseFee; }
    public void setBaseFee(double baseFee) { this.baseFee = baseFee; }
    
    public double getPriorityFee() { return priorityFee; }
    public void setPriorityFee(double priorityFee) { this.priorityFee = priorityFee; }
    
    public double getTotalFee() { return totalFee; }
    public void setTotalFee(double totalFee) { this.totalFee = totalFee; }
    
    public String getCurrency() { return currency; }
    public void setCurrency(String currency) { this.currency = currency; }
    
    public long getEstimatedAt() { return estimatedAt; }
    public void setEstimatedAt(long estimatedAt) { this.estimatedAt = estimatedAt; }
}
