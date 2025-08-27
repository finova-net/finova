// finova-net/finova/mobile-sdk/ios/FinovaSDK/Sources/Utils/Extensions.swift

//
//  Extensions.swift
//  FinovaSDK
//
//  Created by Finova Network
//  Copyright © 2025 Finova Network. All rights reserved.
//

import Foundation
import UIKit
import CryptoKit
import Network

// MARK: - String Extensions
extension String {
    
    /// Validate email format
    var isValidEmail: Bool {
        let emailRegex = #"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"#
        return NSPredicate(format: "SELF MATCHES %@", emailRegex).evaluate(with: self)
    }
    
    /// Validate Solana wallet address
    var isValidSolanaAddress: Bool {
        return self.count == 44 && self.allSatisfy { $0.isLetter || $0.isNumber }
    }
    
    /// Generate referral code from string
    var toReferralCode: String {
        let hash = SHA256.hash(data: self.data(using: .utf8) ?? Data())
        return String(hash.compactMap { String(format: "%02x", $0) }.joined().prefix(8)).uppercased()
    }
    
    /// Format $FIN amount with proper decimals
    func formatAsFIN() -> String {
        guard let value = Double(self) else { return "0.000000 $FIN" }
        return String(format: "%.6f $FIN", value)
    }
    
    /// Truncate wallet address for display
    func truncateWalletAddress() -> String {
        guard self.count > 8 else { return self }
        return "\(self.prefix(4))...\(self.suffix(4))"
    }
    
    /// Remove HTML tags
    var stripHTML: String {
        return self.replacingOccurrences(of: "<[^>]+>", with: "", options: .regularExpression)
    }
    
    /// URL encode for social media integration
    var urlEncoded: String {
        return self.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? self
    }
}

// MARK: - Double Extensions
extension Double {
    
    /// Convert to $FIN display format
    var asFINString: String {
        return String(format: "%.6f $FIN", self)
    }
    
    /// Convert to mining rate format
    var asMiningRate: String {
        return String(format: "%.4f $FIN/hour", self)
    }
    
    /// Convert to percentage format
    var asPercentage: String {
        return String(format: "%.1f%%", self * 100)
    }
    
    /// Round to specific decimal places
    func rounded(toPlaces places: Int) -> Double {
        let divisor = pow(10.0, Double(places))
        return (self * divisor).rounded() / divisor
    }
    
    /// Format large numbers with K, M, B suffixes
    var abbreviated: String {
        let billion = 1_000_000_000.0
        let million = 1_000_000.0
        let thousand = 1_000.0
        
        if abs(self) >= billion {
            return String(format: "%.1fB", self / billion)
        } else if abs(self) >= million {
            return String(format: "%.1fM", self / million)
        } else if abs(self) >= thousand {
            return String(format: "%.1fK", self / thousand)
        } else {
            return String(format: "%.0f", self)
        }
    }
    
    /// Calculate exponential regression factor
    func exponentialRegression(factor: Double = 0.001) -> Double {
        return exp(-factor * self)
    }
}

// MARK: - Int Extensions
extension Int {
    
    /// Convert level to tier name
    var tierName: String {
        switch self {
        case 1...10: return "Bronze"
        case 11...25: return "Silver"
        case 26...50: return "Gold"
        case 51...75: return "Platinum"
        case 76...100: return "Diamond"
        case 101...: return "Mythic"
        default: return "Unranked"
        }
    }
    
    /// Calculate XP required for level
    var xpRequired: Int {
        if self <= 10 {
            return (self - 1) * 100
        } else if self <= 25 {
            return 1000 + (self - 11) * 250
        } else if self <= 50 {
            return 5000 + (self - 26) * 500
        } else if self <= 75 {
            return 20000 + (self - 51) * 1000
        } else if self <= 100 {
            return 50000 + (self - 76) * 2000
        } else {
            return 100000 + (self - 101) * 5000
        }
    }
    
    /// Convert RP to tier
    var rpTier: String {
        switch self {
        case 0...999: return "Explorer"
        case 1000...4999: return "Connector"
        case 5000...14999: return "Influencer"
        case 15000...49999: return "Leader"
        case 50000...: return "Ambassador"
        default: return "Unknown"
        }
    }
    
    /// Format as duration string
    var asDuration: String {
        let hours = self / 3600
        let minutes = (self % 3600) / 60
        let seconds = self % 60
        
        if hours > 0 {
            return String(format: "%02d:%02d:%02d", hours, minutes, seconds)
        } else {
            return String(format: "%02d:%02d", minutes, seconds)
        }
    }
}

// MARK: - Date Extensions
extension Date {
    
    /// Check if date is today
    var isToday: Bool {
        return Calendar.current.isDateInToday(self)
    }
    
    /// Check if date is yesterday
    var isYesterday: Bool {
        return Calendar.current.isDateInYesterday(self)
    }
    
    /// Get days since date
    var daysSince: Int {
        return Calendar.current.dateComponents([.day], from: self, to: Date()).day ?? 0
    }
    
    /// Get hours since date
    var hoursSince: Int {
        return Calendar.current.dateComponents([.hour], from: self, to: Date()).hour ?? 0
    }
    
    /// Format as mining timestamp
    var asMiningTimestamp: String {
        let formatter = DateFormatter()
        formatter.dateFormat = "MMM dd, HH:mm"
        return formatter.string(from: self)
    }
    
    /// Format as relative time
    var asRelativeTime: String {
        let now = Date()
        let components = Calendar.current.dateComponents([.minute, .hour, .day], from: self, to: now)
        
        if let days = components.day, days > 0 {
            return "\(days)d ago"
        } else if let hours = components.hour, hours > 0 {
            return "\(hours)h ago"
        } else if let minutes = components.minute, minutes > 0 {
            return "\(minutes)m ago"
        } else {
            return "Just now"
        }
    }
    
    /// Check if within streak window
    func isWithinStreakWindow() -> Bool {
        let calendar = Calendar.current
        let now = Date()
        let startOfToday = calendar.startOfDay(for: now)
        let startOfYesterday = calendar.date(byAdding: .day, value: -1, to: startOfToday)!
        
        return self >= startOfYesterday
    }
}

// MARK: - Data Extensions
extension Data {
    
    /// Convert to hex string
    var hexString: String {
        return self.map { String(format: "%02x", $0) }.joined()
    }
    
    /// Create secure hash
    var sha256Hash: String {
        let hash = SHA256.hash(data: self)
        return hash.compactMap { String(format: "%02x", $0) }.joined()
    }
    
    /// Convert to base58 string (for Solana addresses)
    var base58String: String {
        let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
        var num = BigUInt(self)
        var result = ""
        
        while num > 0 {
            let remainder = num % 58
            result = String(alphabet[alphabet.index(alphabet.startIndex, offsetBy: Int(remainder))]) + result
            num /= 58
        }
        
        // Add leading zeros
        for byte in self {
            if byte == 0 {
                result = "1" + result
            } else {
                break
            }
        }
        
        return result
    }
}

// MARK: - UIView Extensions
extension UIView {
    
    /// Add gradient background
    func addGradient(colors: [UIColor], startPoint: CGPoint = CGPoint(x: 0, y: 0), endPoint: CGPoint = CGPoint(x: 1, y: 1)) {
        let gradient = CAGradientLayer()
        gradient.colors = colors.map { $0.cgColor }
        gradient.startPoint = startPoint
        gradient.endPoint = endPoint
        gradient.frame = self.bounds
        self.layer.insertSublayer(gradient, at: 0)
    }
    
    /// Add shadow effect
    func addShadow(color: UIColor = .black, opacity: Float = 0.3, radius: CGFloat = 5, offset: CGSize = CGSize(width: 0, height: 2)) {
        self.layer.shadowColor = color.cgColor
        self.layer.shadowOpacity = opacity
        self.layer.shadowRadius = radius
        self.layer.shadowOffset = offset
        self.layer.masksToBounds = false
    }
    
    /// Add border
    func addBorder(color: UIColor, width: CGFloat) {
        self.layer.borderColor = color.cgColor
        self.layer.borderWidth = width
    }
    
    /// Make circular
    func makeCircular() {
        self.layer.cornerRadius = min(self.frame.width, self.frame.height) / 2
        self.clipsToBounds = true
    }
    
    /// Add corner radius
    func addCornerRadius(_ radius: CGFloat) {
        self.layer.cornerRadius = radius
        self.clipsToBounds = true
    }
    
    /// Animate pulse effect
    func pulseAnimation() {
        let pulse = CABasicAnimation(keyPath: "transform.scale")
        pulse.duration = 0.6
        pulse.fromValue = 1.0
        pulse.toValue = 1.1
        pulse.autoreverses = true
        pulse.repeatCount = .infinity
        self.layer.add(pulse, forKey: "pulse")
    }
    
    /// Stop all animations
    func stopAnimations() {
        self.layer.removeAllAnimations()
    }
}

// MARK: - UIColor Extensions
extension UIColor {
    
    /// Finova brand colors
    static let finovaPrimary = UIColor(red: 0.2, green: 0.6, blue: 1.0, alpha: 1.0)
    static let finovaSecondary = UIColor(red: 0.8, green: 0.3, blue: 0.9, alpha: 1.0)
    static let finovaAccent = UIColor(red: 1.0, green: 0.8, blue: 0.2, alpha: 1.0)
    static let finovaSuccess = UIColor(red: 0.2, green: 0.8, blue: 0.4, alpha: 1.0)
    static let finovaWarning = UIColor(red: 1.0, green: 0.6, blue: 0.2, alpha: 1.0)
    static let finovaError = UIColor(red: 1.0, green: 0.3, blue: 0.3, alpha: 1.0)
    
    /// Mining tier colors
    static let bronzeTier = UIColor(red: 0.8, green: 0.5, blue: 0.2, alpha: 1.0)
    static let silverTier = UIColor(red: 0.75, green: 0.75, blue: 0.75, alpha: 1.0)
    static let goldTier = UIColor(red: 1.0, green: 0.84, blue: 0.0, alpha: 1.0)
    static let platinumTier = UIColor(red: 0.9, green: 0.9, blue: 0.98, alpha: 1.0)
    static let diamondTier = UIColor(red: 0.7, green: 0.9, blue: 1.0, alpha: 1.0)
    static let mythicTier = UIColor(red: 0.6, green: 0.0, blue: 0.8, alpha: 1.0)
    
    /// Create color from hex string
    convenience init(hex: String) {
        let hex = hex.trimmingCharacters(in: CharacterSet.alphanumerics.inverted)
        var int: UInt64 = 0
        Scanner(string: hex).scanHexInt64(&int)
        let a, r, g, b: UInt64
        switch hex.count {
        case 3: // RGB (12-bit)
            (a, r, g, b) = (255, (int >> 8) * 17, (int >> 4 & 0xF) * 17, (int & 0xF) * 17)
        case 6: // RGB (24-bit)
            (a, r, g, b) = (255, int >> 16, int >> 8 & 0xFF, int & 0xFF)
        case 8: // ARGB (32-bit)
            (a, r, g, b) = (int >> 24, int >> 16 & 0xFF, int >> 8 & 0xFF, int & 0xFF)
        default:
            (a, r, g, b) = (1, 1, 1, 0)
        }
        
        self.init(
            red: Double(r) / 255,
            green: Double(g) / 255,
            blue: Double(b) / 255,
            alpha: Double(a) / 255
        )
    }
    
    /// Get tier color by level
    static func tierColor(for level: Int) -> UIColor {
        switch level {
        case 1...10: return .bronzeTier
        case 11...25: return .silverTier
        case 26...50: return .goldTier
        case 51...75: return .platinumTier
        case 76...100: return .diamondTier
        case 101...: return .mythicTier
        default: return .systemGray
        }
    }
}

// MARK: - URLRequest Extensions
extension URLRequest {
    
    /// Add Finova API headers
    mutating func addFinovaHeaders(token: String? = nil) {
        self.setValue("application/json", forHTTPHeaderField: "Content-Type")
        self.setValue("FinovaSDK/1.0", forHTTPHeaderField: "User-Agent")
        self.setValue("ios", forHTTPHeaderField: "X-Platform")
        self.setValue(UIDevice.current.systemVersion, forHTTPHeaderField: "X-iOS-Version")
        
        if let token = token {
            self.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }
    }
    
    /// Add device fingerprint for security
    mutating func addDeviceFingerprint() {
        let device = UIDevice.current
        let fingerprint = [
            device.identifierForVendor?.uuidString ?? "unknown",
            device.model,
            device.systemName,
            device.systemVersion
        ].joined(separator: "|")
        
        let hash = SHA256.hash(data: fingerprint.data(using: .utf8) ?? Data())
        let hashString = hash.compactMap { String(format: "%02x", $0) }.joined()
        
        self.setValue(hashString, forHTTPHeaderField: "X-Device-Fingerprint")
    }
}

// MARK: - Array Extensions
extension Array {
    
    /// Safe subscript access
    subscript(safe index: Int) -> Element? {
        return indices.contains(index) ? self[index] : nil
    }
    
    /// Chunk array into smaller arrays
    func chunked(into size: Int) -> [[Element]] {
        return stride(from: 0, to: count, by: size).map {
            Array(self[$0..<Swift.min($0 + size, count)])
        }
    }
}

extension Array where Element: Hashable {
    
    /// Remove duplicates while preserving order
    func removingDuplicates() -> [Element] {
        var seen = Set<Element>()
        return filter { seen.insert($0).inserted }
    }
}

// MARK: - UserDefaults Extensions
extension UserDefaults {
    
    /// Finova-specific keys
    enum FinovaKeys {
        static let userToken = "finova_user_token"
        static let walletAddress = "finova_wallet_address"
        static let lastMiningClaim = "finova_last_mining_claim"
        static let dailyStreak = "finova_daily_streak"
        static let notificationsEnabled = "finova_notifications_enabled"
        static let biometricEnabled = "finova_biometric_enabled"
        static let appVersion = "finova_app_version"
        static let onboardingCompleted = "finova_onboarding_completed"
    }
    
    /// Set Finova-specific values
    func setFinova<T>(_ value: T, forKey key: String) {
        self.set(value, forKey: key)
    }
    
    /// Get Finova-specific values
    func getFinova<T>(_ type: T.Type, forKey key: String) -> T? {
        return self.object(forKey: key) as? T
    }
    
    /// Clear all Finova data
    func clearFinovaData() {
        let keys = [
            FinovaKeys.userToken,
            FinovaKeys.walletAddress,
            FinovaKeys.lastMiningClaim,
            FinovaKeys.dailyStreak,
            FinovaKeys.notificationsEnabled,
            FinovaKeys.biometricEnabled
        ]
        
        keys.forEach { removeObject(forKey: $0) }
    }
}

// MARK: - Notification Extensions
extension Notification.Name {
    static let finovaMiningUpdated = Notification.Name("finova_mining_updated")
    static let finovaXPGained = Notification.Name("finova_xp_gained")
    static let finovaRPUpdated = Notification.Name("finova_rp_updated")
    static let finovaLevelUp = Notification.Name("finova_level_up")
    static let finovaNetworkChanged = Notification.Name("finova_network_changed")
    static let finovaWalletConnected = Notification.Name("finova_wallet_connected")
    static let finovaLoggedOut = Notification.Name("finova_logged_out")
}

// MARK: - Error Extensions
extension Error {
    
    /// Get user-friendly error message
    var localizedMessage: String {
        if let finovaError = self as? FinovaError {
            return finovaError.localizedDescription
        }
        return self.localizedDescription
    }
    
    /// Check if error is network-related
    var isNetworkError: Bool {
        let nsError = self as NSError
        return nsError.domain == NSURLErrorDomain
    }
    
    /// Check if error requires authentication
    var requiresReauth: Bool {
        let nsError = self as NSError
        return nsError.code == 401 || nsError.code == 403
    }
}

// MARK: - BigUInt Helper (Simplified)
private struct BigUInt {
    private var data: [UInt8]
    
    init(_ data: Data) {
        self.data = Array(data)
    }
    
    static func >(lhs: BigUInt, rhs: Int) -> Bool {
        return lhs.data.contains { $0 > 0 }
    }
    
    static func %(lhs: BigUInt, rhs: Int) -> Int {
        // Simplified modulo operation
        var remainder = 0
        for byte in lhs.data {
            remainder = (remainder * 256 + Int(byte)) % rhs
        }
        return remainder
    }
    
    static func /=(lhs: inout BigUInt, rhs: Int) {
        // Simplified division operation
        var carry = 0
        for i in 0..<lhs.data.count {
            let temp = carry * 256 + Int(lhs.data[i])
            lhs.data[i] = UInt8(temp / rhs)
            carry = temp % rhs
        }
    }
}

// MARK: - Security Extensions
extension String {
    
    /// Validate content for security
    var isSafeContent: Bool {
        let dangerousPatterns = [
            "<script", "javascript:", "data:text/html",
            "onload=", "onerror=", "onclick="
        ]
        
        let lowercased = self.lowercased()
        return !dangerousPatterns.contains { lowercased.contains($0) }
    }
    
    /// Sanitize user input
    var sanitized: String {
        return self
            .replacingOccurrences(of: "<", with: "&lt;")
            .replacingOccurrences(of: ">", with: "&gt;")
            .replacingOccurrences(of: "\"", with: "&quot;")
            .replacingOccurrences(of: "'", with: "&#x27;")
            .replacingOccurrences(of: "&", with: "&amp;")
    }
}
