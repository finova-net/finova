# finova-net/finova/mobile-sdk/ios/FinovaSDK/README.md

# Finova iOS SDK

[![Swift Version](https://img.shields.io/badge/Swift-5.9+-orange.svg)](https://swift.org)
[![iOS Version](https://img.shields.io/badge/iOS-14.0+-blue.svg)](https://developer.apple.com/ios/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![CocoaPods](https://img.shields.io/cocoapods/v/FinovaSDK.svg)](https://cocoapods.org/pods/FinovaSDK)
[![SPM Compatible](https://img.shields.io/badge/SPM-compatible-brightgreen.svg)](https://swift.org/package-manager/)

The official iOS SDK for Finova Network - the next-generation Social-Fi Super App that integrates XP, RP, and $FIN mining systems.

## Features

- 🎯 **Triple Reward System**: XP, RP, and $FIN mining integration
- ⛏️ **Real-time Mining**: Background mining with exponential regression
- 📱 **Social Integration**: Connect Instagram, TikTok, YouTube, Facebook, X
- 🎮 **Gamification**: Hamster Kombat-inspired progression system
- 🔗 **Referral Network**: Multi-level referral tracking and rewards
- 🎴 **NFT System**: Special cards and collectibles
- 🏛️ **DAO Governance**: Decentralized voting and proposals
- 🔒 **Enterprise Security**: Biometric auth, anti-bot protection

## Installation

### Swift Package Manager (Recommended)

```swift
dependencies: [
    .package(url: "https://github.com/finova-network/ios-sdk.git", from: "1.0.0")
]
```

### CocoaPods

```ruby
pod 'FinovaSDK', '~> 1.0'
```

### Manual Installation

1. Download the latest release from [GitHub Releases](https://github.com/finova-network/ios-sdk/releases)
2. Drag `FinovaSDK.xcframework` into your Xcode project

## Quick Start

### 1. Initialize SDK

```swift
import FinovaSDK

class AppDelegate: UIResponder, UIApplicationDelegate {
    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        
        // Initialize Finova SDK
        FinovaSDK.configure(
            apiKey: "your_api_key",
            environment: .production, // .staging, .development
            enableBiometrics: true,
            enableBackgroundMining: true
        )
        
        return true
    }
}
```

### 2. User Authentication

```swift
import FinovaSDK

class LoginViewController: UIViewController {
    
    @IBAction func loginTapped(_ sender: UIButton) {
        // Social login with referral code
        FinovaClient.shared.authenticate(
            provider: .google,
            referralCode: "FINOVA2025"
        ) { [weak self] result in
            switch result {
            case .success(let user):
                print("Login successful: \(user.username)")
                self?.navigateToMain()
            case .failure(let error):
                self?.showError(error.localizedDescription)
            }
        }
    }
    
    @IBAction func biometricLoginTapped(_ sender: UIButton) {
        // Biometric authentication
        FinovaClient.shared.authenticateWithBiometrics { result in
            switch result {
            case .success:
                print("Biometric auth successful")
            case .failure(let error):
                print("Biometric auth failed: \(error)")
            }
        }
    }
}
```

### 3. Start Mining

```swift
import FinovaSDK

class MiningViewController: UIViewController {
    @IBOutlet weak var miningRateLabel: UILabel!
    @IBOutlet weak var balanceLabel: UILabel!
    @IBOutlet weak var miningButton: UIButton!
    
    override func viewDidLoad() {
        super.viewDidLoad()
        setupMiningObserver()
    }
    
    @IBAction func startMiningTapped(_ sender: UIButton) {
        MiningService.shared.startMining { [weak self] result in
            DispatchQueue.main.async {
                switch result {
                case .success(let miningData):
                    self?.updateMiningUI(miningData)
                case .failure(let error):
                    self?.showError(error.localizedDescription)
                }
            }
        }
    }
    
    private func setupMiningObserver() {
        MiningService.shared.onMiningUpdate = { [weak self] miningData in
            DispatchQueue.main.async {
                self?.miningRateLabel.text = "\(miningData.currentRate) $FIN/hour"
                self?.balanceLabel.text = "\(miningData.totalBalance) $FIN"
            }
        }
    }
}
```

### 4. Social Platform Integration

```swift
import FinovaSDK

class SocialViewController: UIViewController {
    
    @IBAction func connectInstagramTapped(_ sender: UIButton) {
        FinovaClient.shared.connectSocialPlatform(.instagram) { result in
            switch result {
            case .success(let platform):
                print("Connected to \(platform.name)")
                self.trackSocialActivity()
            case .failure(let error):
                print("Connection failed: \(error)")
            }
        }
    }
    
    private func trackSocialActivity() {
        // Track user's social media posts for XP
        XPService.shared.trackActivity(
            type: .originalPost,
            platform: .instagram,
            content: "Check out this amazing view! #finova",
            mediaType: .photo
        ) { result in
            switch result {
            case .success(let xpGain):
                print("Earned \(xpGain.amount) XP!")
            case .failure(let error):
                print("XP tracking failed: \(error)")
            }
        }
    }
}
```

### 5. XP System Usage

```swift
import FinovaSDK

class XPViewController: UIViewController {
    @IBOutlet weak var levelLabel: UILabel!
    @IBOutlet weak var xpProgressView: UIProgressView!
    @IBOutlet weak var multiplierLabel: UILabel!
    
    override func viewDidLoad() {
        super.viewDidLoad()
        loadUserXP()
    }
    
    private func loadUserXP() {
        XPService.shared.getUserXP { [weak self] result in
            DispatchQueue.main.async {
                switch result {
                case .success(let xpData):
                    self?.updateXPUI(xpData)
                case .failure(let error):
                    print("Failed to load XP: \(error)")
                }
            }
        }
    }
    
    private func updateXPUI(_ xpData: XPData) {
        levelLabel.text = "Level \(xpData.level)"
        xpProgressView.progress = Float(xpData.currentXP) / Float(xpData.nextLevelXP)
        multiplierLabel.text = "\(xpData.miningMultiplier)x Mining Boost"
    }
    
    // Post content and earn XP
    @IBAction func postContentTapped(_ sender: UIButton) {
        let activity = XPActivity(
            type: .originalPost,
            platform: .tiktok,
            content: "Dancing to the latest trend! #viral",
            mediaType: .video,
            engagementScore: 0.85
        )
        
        XPService.shared.submitActivity(activity) { result in
            switch result {
            case .success(let xpReward):
                print("Earned \(xpReward.baseXP) XP with \(xpReward.multiplier)x multiplier")
            case .failure(let error):
                print("XP submission failed: \(error)")
            }
        }
    }
}
```

### 6. Referral System

```swift
import FinovaSDK

class ReferralViewController: UIViewController {
    @IBOutlet weak var referralCodeLabel: UILabel!
    @IBOutlet weak var networkSizeLabel: UILabel!
    @IBOutlet weak var rpBalanceLabel: UILabel!
    
    override func viewDidLoad() {
        super.viewDidLoad()
        loadReferralData()
    }
    
    private func loadReferralData() {
        ReferralService.shared.getReferralData { [weak self] result in
            DispatchQueue.main.async {
                switch result {
                case .success(let referralData):
                    self?.updateReferralUI(referralData)
                case .failure(let error):
                    print("Failed to load referral data: \(error)")
                }
            }
        }
    }
    
    private func updateReferralUI(_ data: ReferralData) {
        referralCodeLabel.text = data.referralCode
        networkSizeLabel.text = "\(data.totalReferrals) Referrals"
        rpBalanceLabel.text = "\(data.rpBalance) RP"
    }
    
    @IBAction func shareReferralTapped(_ sender: UIButton) {
        ReferralService.shared.generateReferralLink { result in
            switch result {
            case .success(let link):
                let activityVC = UIActivityViewController(
                    activityItems: [link],
                    applicationActivities: nil
                )
                self.present(activityVC, animated: true)
            case .failure(let error):
                print("Failed to generate referral link: \(error)")
            }
        }
    }
}
```

### 7. NFT Special Cards

```swift
import FinovaSDK

class NFTViewController: UIViewController {
    @IBOutlet weak var collectionView: UICollectionView!
    
    private var nftCards: [NFTCard] = []
    
    override func viewDidLoad() {
        super.viewDidLoad()
        loadUserNFTs()
    }
    
    private func loadUserNFTs() {
        NFTService.shared.getUserNFTs { [weak self] result in
            DispatchQueue.main.async {
                switch result {
                case .success(let cards):
                    self?.nftCards = cards
                    self?.collectionView.reloadData()
                case .failure(let error):
                    print("Failed to load NFTs: \(error)")
                }
            }
        }
    }
    
    // Use special card
    func useSpecialCard(_ card: NFTCard) {
        NFTService.shared.useSpecialCard(card.id) { result in
            switch result {
            case .success(let effect):
                print("Card activated: \(effect.description)")
                // Update UI to show active effect
                self.showCardEffect(effect)
            case .failure(let error):
                print("Failed to use card: \(error)")
            }
        }
    }
    
    private func showCardEffect(_ effect: CardEffect) {
        let alert = UIAlertController(
            title: "Card Activated!",
            message: "\(effect.name): +\(effect.bonus)% for \(effect.duration)",
            preferredStyle: .alert
        )
        alert.addAction(UIAlertAction(title: "OK", style: .default))
        present(alert, animated: true)
    }
}
```

### 8. Staking Integration

```swift
import FinovaSDK

class StakingViewController: UIViewController {
    @IBOutlet weak var stakingAmountTextField: UITextField!
    @IBOutlet weak var apyLabel: UILabel!
    @IBOutlet weak var stakingBalanceLabel: UILabel!
    
    @IBAction func stakeTapped(_ sender: UIButton) {
        guard let amountText = stakingAmountTextField.text,
              let amount = Double(amountText) else { return }
        
        FinovaClient.shared.stakeTokens(amount: amount) { result in
            DispatchQueue.main.async {
                switch result {
                case .success(let stakingInfo):
                    self.updateStakingUI(stakingInfo)
                case .failure(let error):
                    self.showError(error.localizedDescription)
                }
            }
        }
    }
    
    private func updateStakingUI(_ stakingInfo: StakingInfo) {
        stakingBalanceLabel.text = "\(stakingInfo.stakedAmount) $sFIN"
        apyLabel.text = "\(stakingInfo.currentAPY)% APY"
    }
}
```

## Advanced Features

### Background Mining

```swift
// Enable background mining (requires Background App Refresh)
MiningService.shared.enableBackgroundMining(true)

// Monitor mining status
MiningService.shared.onMiningStatusChanged = { status in
    switch status {
    case .active:
        print("Mining is active")
    case .paused:
        print("Mining is paused")
    case .stopped:
        print("Mining stopped")
    }
}
```

### Real-time WebSocket Updates

```swift
// Connect to real-time updates
FinovaClient.shared.connectWebSocket { result in
    switch result {
    case .success:
        print("WebSocket connected")
    case .failure(let error):
        print("WebSocket connection failed: \(error)")
    }
}

// Listen for real-time events
FinovaClient.shared.onRealtimeEvent = { event in
    switch event.type {
    case .miningUpdate:
        // Handle mining rate changes
        break
    case .xpGained:
        // Handle XP notifications
        break
    case .referralReward:
        // Handle referral earnings
        break
    case .nftReceived:
        // Handle NFT rewards
        break
    }
}
```

### Analytics Integration

```swift
import FinovaSDK

// Track custom events
AnalyticsService.shared.track("user_action", parameters: [
    "action_type": "special_card_used",
    "card_id": "mining_boost_rare",
    "user_level": 25
])

// Track user progression
AnalyticsService.shared.trackLevelUp(
    newLevel: 30,
    xpGained: 1500,
    timeTaken: 86400 // seconds
)
```

## Configuration

### Environment Setup

```swift
// Development
FinovaSDK.configure(
    apiKey: "dev_api_key",
    environment: .development,
    baseURL: "https://api-dev.finova.network",
    solanaCluster: .devnet
)

// Production
FinovaSDK.configure(
    apiKey: "prod_api_key",
    environment: .production,
    baseURL: "https://api.finova.network",
    solanaCluster: .mainnet
)
```

### Custom Configuration

```swift
let config = FinovaConfiguration(
    apiKey: "your_api_key",
    environment: .production,
    enableBiometrics: true,
    enableBackgroundMining: true,
    miningInterval: 3600, // 1 hour
    maxDailyXP: 10000,
    enableAnalytics: true,
    debugMode: false
)

FinovaSDK.configure(with: config)
```

## Security

### Biometric Authentication

```swift
// Check biometric availability
if BiometricService.shared.isBiometricAvailable {
    // Enable biometric login
    BiometricService.shared.enableBiometricAuth()
}

// Custom biometric prompt
BiometricService.shared.authenticate(
    reason: "Authenticate to access your Finova wallet"
) { result in
    switch result {
    case .success:
        // Proceed with sensitive operation
        break
    case .failure(let error):
        // Handle authentication failure
        break
    }
}
```

### Secure Storage

```swift
// Store sensitive data securely
SecureStorage.shared.store("user_private_key", value: privateKey)

// Retrieve secure data
let privateKey = SecureStorage.shared.retrieve("user_private_key")
```

## Error Handling

```swift
// Handle specific Finova errors
FinovaClient.shared.startMining { result in
    switch result {
    case .success(let miningData):
        // Handle success
        break
    case .failure(let error):
        if let finovaError = error as? FinovaError {
            switch finovaError {
            case .networkError:
                // Handle network issues
                break
            case .authenticationFailed:
                // Redirect to login
                break
            case .insufficientBalance:
                // Show balance error
                break
            case .rateLimitExceeded:
                // Show rate limit message
                break
            case .botDetected:
                // Handle anti-bot measures
                break
            }
        }
    }
}
```

## Testing

### Unit Testing

```swift
import XCTest
@testable import FinovaSDK

class FinovaSDKTests: XCTestCase {
    
    func testMiningCalculation() {
        let mining = MiningService.shared
        let rate = mining.calculateMiningRate(
            baseRate: 0.1,
            userLevel: 25,
            referralCount: 10,
            stakingAmount: 1000
        )
        
        XCTAssertGreaterThan(rate, 0.1)
        XCTAssertLessThan(rate, 0.5)
    }
    
    func testXPCalculation() {
        let xp = XPService.shared
        let gain = xp.calculateXPGain(
            activityType: .originalPost,
            platform: .instagram,
            qualityScore: 1.5,
            userLevel: 20
        )
        
        XCTAssertEqual(gain, 75) // 50 * 1.5 * platform_multiplier
    }
}
```

### Integration Testing

```swift
func testFullUserJourney() {
    let expectation = XCTestExpectation(description: "Complete user journey")
    
    // 1. Register user
    FinovaClient.shared.register(
        username: "testuser",
        email: "test@example.com"
    ) { result in
        XCTAssertTrue(result.isSuccess)
        
        // 2. Start mining
        MiningService.shared.startMining { result in
            XCTAssertTrue(result.isSuccess)
            
            // 3. Submit XP activity
            XPService.shared.submitActivity(testActivity) { result in
                XCTAssertTrue(result.isSuccess)
                expectation.fulfill()
            }
        }
    }
    
    wait(for: [expectation], timeout: 10.0)
}
```

## Requirements

- iOS 14.0+
- Xcode 14.0+
- Swift 5.9+

## Dependencies

- **Alamofire**: Network requests
- **Starscream**: WebSocket connections
- **Solana.Swift**: Blockchain integration
- **KeychainAccess**: Secure storage
- **LocalAuthentication**: Biometric auth

## Support

- 📧 **Email**: sdk-support@finova.network
- 💬 **Discord**: [Finova Community](https://discord.gg/finova)
- 📖 **Documentation**: [docs.finova.network](https://docs.finova.network)
- 🐛 **Issues**: [GitHub Issues](https://github.com/finova-network/ios-sdk/issues)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

---

**Start building the future of Social-Fi with Finova iOS SDK! 🚀**
