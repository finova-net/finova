// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Utils/Validation.swift

//
//  Validation.swift
//  FinovaSDK
//
//  Created by Finova Network Team
//  Copyright © 2025 Finova Network. All rights reserved.
//

import Foundation
import CryptoKit
import Network

/// Comprehensive validation utilities for Finova Network iOS SDK
/// Handles wallet addresses, mining rates, XP calculations, RP validation, and security checks
public final class FinovaValidation {
    
    // MARK: - Constants
    
    private struct ValidationConstants {
        static let solanaAddressLength = 44
        static let solanaAddressCharacterSet = CharacterSet(charactersIn: "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz")
        static let maxMiningRate: Double = 15.0 // Max \$FIN per day
        static let maxXPPerActivity = 2000
        static let maxRPPerAction = 15000
        static let minKYCAge = 18
        static let maxUsernameLength = 32
        static let minPasswordLength = 8
        static let maxReferralCodeLength = 16
        static let socialPlatforms = ["instagram", "tiktok", "youtube", "facebook", "twitter", "x"]
    }
    
    // MARK: - Wallet & Address Validation
    
    /// Validates Solana wallet address format
    /// - Parameter address: Wallet address string
    /// - Returns: True if valid Solana address
    public static func isValidSolanaAddress(_ address: String) -> Bool {
        guard address.count == ValidationConstants.solanaAddressLength else { return false }
        
        let addressCharacterSet = CharacterSet(charactersIn: address)
        return ValidationConstants.solanaAddressCharacterSet.isSuperset(of: addressCharacterSet)
    }
    
    /// Validates token mint address
    /// - Parameter mintAddress: Token mint address
    /// - Returns: Validation result with details
    public static func validateTokenMint(_ mintAddress: String) -> ValidationResult {
        guard !mintAddress.isEmpty else {
            return ValidationResult(isValid: false, error: .emptyValue, message: "Token mint address cannot be empty")
        }
        
        guard isValidSolanaAddress(mintAddress) else {
            return ValidationResult(isValid: false, error: .invalidFormat, message: "Invalid token mint address format")
        }
        
        return ValidationResult(isValid: true, error: nil, message: "Valid token mint address")
    }
    
    // MARK: - Mining Validation
    
    /// Validates mining rate calculation
    /// - Parameters:
    ///   - baseRate: Base mining rate
    ///   - userLevel: User XP level
    ///   - rpTier: Referral Points tier
    ///   - stakingMultiplier: Staking bonus multiplier
    /// - Returns: Validation result with calculated rate
    public static func validateMiningRate(
        baseRate: Double,
        userLevel: Int,
        rpTier: Int,
        stakingMultiplier: Double = 1.0
    ) -> MiningValidationResult {
        
        // Base rate validation
        guard baseRate > 0 && baseRate <= 0.1 else {
            return MiningValidationResult(
                isValid: false,
                finalRate: 0,
                error: .invalidMiningRate,
                message: "Base mining rate must be between 0 and 0.1 \$FIN/hour"
            )
        }
        
        // XP level validation
        guard userLevel >= 1 && userLevel <= 200 else {
            return MiningValidationResult(
                isValid: false,
                finalRate: 0,
                error: .invalidLevel,
                message: "User level must be between 1 and 200"
            )
        }
        
        // RP tier validation
        guard rpTier >= 0 && rpTier <= 5 else {
            return MiningValidationResult(
                isValid: false,
                finalRate: 0,
                error: .invalidRPTier,
                message: "RP tier must be between 0 and 5"
            )
        }
        
        // Calculate mining rate using whitepaper formula
        let xpMultiplier = 1.0 + (Double(userLevel) / 100.0)
        let rpMultiplier = 1.0 + (Double(rpTier) * 0.2)
        let finalRate = baseRate * xpMultiplier * rpMultiplier * stakingMultiplier
        
        // Check against daily maximum
        let dailyRate = finalRate * 24
        guard dailyRate <= ValidationConstants.maxMiningRate else {
            return MiningValidationResult(
                isValid: false,
                finalRate: finalRate,
                error: .exceededDailyLimit,
                message: "Mining rate exceeds daily limit of \(ValidationConstants.maxMiningRate) \$FIN"
            )
        }
        
        return MiningValidationResult(
            isValid: true,
            finalRate: finalRate,
            error: nil,
            message: "Valid mining rate calculated"
        )
    }
    
    /// Validates exponential regression factor
    /// - Parameter totalHoldings: User's total \$FIN holdings
    /// - Returns: Regression factor (0.1 to 1.0)
    public static func calculateRegressionFactor(totalHoldings: Double) -> Double {
        guard totalHoldings >= 0 else { return 1.0 }
        
        let regressionFactor = exp(-0.001 * totalHoldings)
        return max(0.1, min(1.0, regressionFactor))
    }
    
    // MARK: - XP System Validation
    
    /// Validates XP gain calculation
    /// - Parameters:
    ///   - activityType: Type of social activity
    ///   - platform: Social media platform
    ///   - qualityScore: AI-assessed content quality (0.5-2.0)
    ///   - userLevel: Current user level
    /// - Returns: XP validation result
    public static func validateXPGain(
        activityType: ActivityType,
        platform: String,
        qualityScore: Double,
        userLevel: Int
    ) -> XPValidationResult {
        
        // Platform validation
        guard ValidationConstants.socialPlatforms.contains(platform.lowercased()) else {
            return XPValidationResult(
                isValid: false,
                xpGained: 0,
                error: .unsupportedPlatform,
                message: "Unsupported social media platform: \(platform)"
            )
        }
        
        // Quality score validation
        guard qualityScore >= 0.5 && qualityScore <= 2.0 else {
            return XPValidationResult(
                isValid: false,
                xpGained: 0,
                error: .invalidQualityScore,
                message: "Quality score must be between 0.5 and 2.0"
            )
        }
        
        // Get base XP for activity
        let baseXP = getBaseXP(for: activityType)
        
        // Calculate platform multiplier
        let platformMultiplier = getPlatformMultiplier(for: platform)
        
        // Calculate level progression (exponential decay)
        let levelProgression = exp(-0.01 * Double(userLevel))
        
        // Final XP calculation
        let finalXP = Int(Double(baseXP) * platformMultiplier * qualityScore * levelProgression)
        
        // Check against maximum
        guard finalXP <= ValidationConstants.maxXPPerActivity else {
            return XPValidationResult(
                isValid: false,
                xpGained: finalXP,
                error: .exceededMaxXP,
                message: "XP gain exceeds maximum of \(ValidationConstants.maxXPPerActivity)"
            )
        }
        
        return XPValidationResult(
            isValid: true,
            xpGained: finalXP,
            error: nil,
            message: "Valid XP calculation"
        )
    }
    
    // MARK: - Referral Points Validation
    
    /// Validates referral network structure
    /// - Parameters:
    ///   - directReferrals: Number of direct referrals
    ///   - activeReferrals: Number of active referrals (last 30 days)
    ///   - networkSize: Total network size (L1+L2+L3)
    /// - Returns: RP validation result
    public static func validateReferralNetwork(
        directReferrals: Int,
        activeReferrals: Int,
        networkSize: Int
    ) -> RPValidationResult {
        
        // Basic validation
        guard directReferrals >= 0 && activeReferrals >= 0 && networkSize >= 0 else {
            return RPValidationResult(
                isValid: false,
                rpValue: 0,
                tier: 0,
                error: .negativeValues,
                message: "Referral counts cannot be negative"
            )
        }
        
        guard activeReferrals <= directReferrals else {
            return RPValidationResult(
                isValid: false,
                rpValue: 0,
                tier: 0,
                error: .invalidReferralCount,
                message: "Active referrals cannot exceed direct referrals"
            )
        }
        
        guard networkSize >= directReferrals else {
            return RPValidationResult(
                isValid: false,
                rpValue: 0,
                tier: 0,
                error: .invalidNetworkSize,
                message: "Network size cannot be less than direct referrals"
            )
        }
        
        // Calculate network quality score
        let networkQuality = directReferrals > 0 ? Double(activeReferrals) / Double(directReferrals) : 0.0
        
        // Calculate RP value using whitepaper formula
        let baseRP = Double(activeReferrals * 100) // Base points per active referral
        let networkBonus = Double(networkSize) * 0.1 // Network effect bonus
        let qualityBonus = networkQuality * 1000 // Quality multiplier
        
        let totalRP = Int(baseRP + networkBonus + qualityBonus)
        
        // Determine RP tier
        let rpTier = calculateRPTier(rpValue: totalRP)
        
        // Apply regression for large networks
        let regressionFactor = exp(-0.0001 * Double(networkSize) * networkQuality)
        let finalRP = Int(Double(totalRP) * regressionFactor)
        
        return RPValidationResult(
            isValid: true,
            rpValue: finalRP,
            tier: rpTier,
            error: nil,
            message: "Valid referral network structure"
        )
    }
    
    // MARK: - User Input Validation
    
    /// Validates user registration data
    /// - Parameter userData: User registration data
    /// - Returns: Validation result
    public static func validateUserRegistration(_ userData: UserRegistrationData) -> ValidationResult {
        // Username validation
        guard !userData.username.isEmpty else {
            return ValidationResult(isValid: false, error: .emptyValue, message: "Username cannot be empty")
        }
        
        guard userData.username.count <= ValidationConstants.maxUsernameLength else {
            return ValidationResult(isValid: false, error: .invalidLength, message: "Username too long")
        }
        
        guard userData.username.allSatisfy({ $0.isLetter || $0.isNumber || $0 == "_" }) else {
            return ValidationResult(isValid: false, error: .invalidFormat, message: "Username contains invalid characters")
        }
        
        // Email validation
        guard isValidEmail(userData.email) else {
            return ValidationResult(isValid: false, error: .invalidFormat, message: "Invalid email format")
        }
        
        // Password validation
        guard userData.password.count >= ValidationConstants.minPasswordLength else {
            return ValidationResult(isValid: false, error: .invalidLength, message: "Password too short")
        }
        
        guard hasValidPasswordComplexity(userData.password) else {
            return ValidationResult(isValid: false, error: .weakPassword, message: "Password lacks complexity")
        }
        
        // Age validation for KYC
        if let birthDate = userData.birthDate {
            let age = Calendar.current.dateComponents([.year], from: birthDate, to: Date()).year ?? 0
            guard age >= ValidationConstants.minKYCAge else {
                return ValidationResult(isValid: false, error: .underage, message: "Must be 18+ for KYC verification")
            }
        }
        
        return ValidationResult(isValid: true, error: nil, message: "Valid registration data")
    }
    
    /// Validates referral code format
    /// - Parameter code: Referral code
    /// - Returns: True if valid format
    public static func isValidReferralCode(_ code: String) -> Bool {
        guard !code.isEmpty && code.count <= ValidationConstants.maxReferralCodeLength else { return false }
        return code.allSatisfy { $0.isLetter || $0.isNumber }
    }
    
    // MARK: - Security Validation
    
    /// Validates device fingerprint for anti-bot detection
    /// - Parameter fingerprint: Device fingerprint data
    /// - Returns: Security validation result
    public static func validateDeviceFingerprint(_ fingerprint: DeviceFingerprint) -> SecurityValidationResult {
        var riskScore: Double = 0.0
        var riskFactors: [String] = []
        
        // Check for emulator/simulator
        if fingerprint.isEmulator {
            riskScore += 0.5
            riskFactors.append("Running on emulator")
        }
        
        // Check for jailbreak/root
        if fingerprint.isJailbroken {
            riskScore += 0.3
            riskFactors.append("Device is jailbroken")
        }
        
        // Check device consistency
        if fingerprint.deviceModel.isEmpty || fingerprint.osVersion.isEmpty {
            riskScore += 0.2
            riskFactors.append("Missing device information")
        }
        
        // Check for VPN/Proxy
        if fingerprint.isUsingVPN {
            riskScore += 0.1
            riskFactors.append("Using VPN/Proxy")
        }
        
        // Calculate human probability (inverse of risk)
        let humanProbability = max(0.1, 1.0 - riskScore)
        
        return SecurityValidationResult(
            isValid: riskScore < 0.7,
            humanProbability: humanProbability,
            riskScore: riskScore,
            riskFactors: riskFactors
        )
    }
    
    /// Validates biometric data for KYC
    /// - Parameter biometricData: Face verification data
    /// - Returns: Biometric validation result
    public static func validateBiometricData(_ biometricData: BiometricData) -> BiometricValidationResult {
        guard !biometricData.faceEmbedding.isEmpty else {
            return BiometricValidationResult(
                isValid: false,
                confidenceScore: 0.0,
                error: .missingData,
                message: "Face embedding data missing"
            )
        }
        
        // Basic quality checks
        guard biometricData.qualityScore >= 0.7 else {
            return BiometricValidationResult(
                isValid: false,
                confidenceScore: biometricData.qualityScore,
                error: .lowQuality,
                message: "Biometric quality too low"
            )
        }
        
        // Liveness detection
        guard biometricData.livenessScore >= 0.8 else {
            return BiometricValidationResult(
                isValid: false,
                confidenceScore: biometricData.livenessScore,
                error: .livenessCheckFailed,
                message: "Liveness detection failed"
            )
        }
        
        return BiometricValidationResult(
            isValid: true,
            confidenceScore: min(biometricData.qualityScore, biometricData.livenessScore),
            error: nil,
            message: "Valid biometric data"
        )
    }
    
    // MARK: - Helper Methods
    
    private static func getBaseXP(for activityType: ActivityType) -> Int {
        switch activityType {
        case .textPost: return 50
        case .photoPost: return 75
        case .videoPost: return 150
        case .story: return 25
        case .comment: return 25
        case .like: return 5
        case .share: return 15
        case .follow: return 20
        case .dailyLogin: return 10
        case .viralContent: return 1000
        }
    }
    
    private static func getPlatformMultiplier(for platform: String) -> Double {
        switch platform.lowercased() {
        case "tiktok": return 1.3
        case "youtube": return 1.4
        case "instagram": return 1.2
        case "x", "twitter": return 1.2
        case "facebook": return 1.1
        default: return 1.0
        }
    }
    
    private static func calculateRPTier(rpValue: Int) -> Int {
        switch rpValue {
        case 0..<1000: return 0
        case 1000..<5000: return 1
        case 5000..<15000: return 2
        case 15000..<50000: return 3
        case 50000...: return 4
        default: return 0
        }
    }
    
    private static func isValidEmail(_ email: String) -> Bool {
        let emailRegex = "[A-Z0-9a-z._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,64}"
        let emailPredicate = NSPredicate(format:"SELF MATCHES %@", emailRegex)
        return emailPredicate.evaluate(with: email)
    }
    
    private static func hasValidPasswordComplexity(_ password: String) -> Bool {
        let hasUppercase = password.contains { $0.isUppercase }
        let hasLowercase = password.contains { $0.isLowercase }
        let hasDigit = password.contains { $0.isNumber }
        let hasSpecialChar = password.contains { "!@#$%^&*()_+-=[]{}|;:,.<>?".contains($0) }
        
        return hasUppercase && hasLowercase && hasDigit && hasSpecialChar
    }
}

// MARK: - Data Structures

public struct ValidationResult {
    public let isValid: Bool
    public let error: ValidationError?
    public let message: String
}

public struct MiningValidationResult {
    public let isValid: Bool
    public let finalRate: Double
    public let error: ValidationError?
    public let message: String
}

public struct XPValidationResult {
    public let isValid: Bool
    public let xpGained: Int
    public let error: ValidationError?
    public let message: String
}

public struct RPValidationResult {
    public let isValid: Bool
    public let rpValue: Int
    public let tier: Int
    public let error: ValidationError?
    public let message: String
}

public struct SecurityValidationResult {
    public let isValid: Bool
    public let humanProbability: Double
    public let riskScore: Double
    public let riskFactors: [String]
}

public struct BiometricValidationResult {
    public let isValid: Bool
    public let confidenceScore: Double
    public let error: ValidationError?
    public let message: String
}

public struct UserRegistrationData {
    public let username: String
    public let email: String
    public let password: String
    public let birthDate: Date?
    public let referralCode: String?
    
    public init(username: String, email: String, password: String, birthDate: Date? = nil, referralCode: String? = nil) {
        self.username = username
        self.email = email
        self.password = password
        self.birthDate = birthDate
        self.referralCode = referralCode
    }
}

public struct DeviceFingerprint {
    public let deviceModel: String
    public let osVersion: String
    public let isEmulator: Bool
    public let isJailbroken: Bool
    public let isUsingVPN: Bool
    public let screenResolution: String
    public let timezone: String
    
    public init(deviceModel: String, osVersion: String, isEmulator: Bool, isJailbroken: Bool, isUsingVPN: Bool, screenResolution: String, timezone: String) {
        self.deviceModel = deviceModel
        self.osVersion = osVersion
        self.isEmulator = isEmulator
        self.isJailbroken = isJailbroken
        self.isUsingVPN = isUsingVPN
        self.screenResolution = screenResolution
        self.timezone = timezone
    }
}

public struct BiometricData {
    public let faceEmbedding: [Float]
    public let qualityScore: Double
    public let livenessScore: Double
    public let timestamp: Date
    
    public init(faceEmbedding: [Float], qualityScore: Double, livenessScore: Double, timestamp: Date = Date()) {
        self.faceEmbedding = faceEmbedding
        self.qualityScore = qualityScore
        self.livenessScore = livenessScore
        self.timestamp = timestamp
    }
}

public enum ActivityType {
    case textPost
    case photoPost
    case videoPost
    case story
    case comment
    case like
    case share
    case follow
    case dailyLogin
    case viralContent
}

public enum ValidationError: Error, LocalizedError {
    case emptyValue
    case invalidFormat
    case invalidLength
    case weakPassword
    case underage
    case invalidMiningRate
    case invalidLevel
    case invalidRPTier
    case exceededDailyLimit
    case exceededMaxXP
    case unsupportedPlatform
    case invalidQualityScore
    case negativeValues
    case invalidReferralCount
    case invalidNetworkSize
    case missingData
    case lowQuality
    case livenessCheckFailed
    
    public var errorDescription: String? {
        switch self {
        case .emptyValue: return "Value cannot be empty"
        case .invalidFormat: return "Invalid format"
        case .invalidLength: return "Invalid length"
        case .weakPassword: return "Password is too weak"
        case .underage: return "User must be 18 or older"
        case .invalidMiningRate: return "Invalid mining rate"
        case .invalidLevel: return "Invalid user level"
        case .invalidRPTier: return "Invalid RP tier"
        case .exceededDailyLimit: return "Exceeded daily limit"
        case .exceededMaxXP: return "Exceeded maximum XP"
        case .unsupportedPlatform: return "Unsupported platform"
        case .invalidQualityScore: return "Invalid quality score"
        case .negativeValues: return "Values cannot be negative"
        case .invalidReferralCount: return "Invalid referral count"
        case .invalidNetworkSize: return "Invalid network size"
        case .missingData: return "Required data missing"
        case .lowQuality: return "Quality too low"
        case .livenessCheckFailed: return "Liveness check failed"
        }
    }
}
