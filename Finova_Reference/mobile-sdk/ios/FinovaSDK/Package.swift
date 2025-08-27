// finova-net/finova/mobile-sdk/ios/FinovaSDK/Package.swift

// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "FinovaSDK",
    platforms: [
        .iOS(.v15),
        .macOS(.v12),
        .watchOS(.v8),
        .tvOS(.v15)
    ],
    products: [
        .library(
            name: "FinovaSDK",
            targets: ["FinovaSDK"]
        ),
        .library(
            name: "FinovaCore",
            targets: ["FinovaCore"]
        ),
        .library(
            name: "FinovaMining",
            targets: ["FinovaMining"]
        ),
        .library(
            name: "FinovaXP",
            targets: ["FinovaXP"]
        ),
        .library(
            name: "FinovaReferral",
            targets: ["FinovaReferral"]
        ),
        .library(
            name: "FinovaNFT",
            targets: ["FinovaNFT"]
        ),
        .library(
            name: "FinovaWallet",
            targets: ["FinovaWallet"]
        )
    ],
    dependencies: [
        // Solana SDK for blockchain interactions
        .package(url: "https://github.com/solana-labs/solana-swift.git", from: "1.0.0"),
        
        // Keychain services for secure storage
        .package(url: "https://github.com/evgenyneu/keychain-swift.git", from: "20.0.0"),
        
        // Networking
        .package(url: "https://github.com/Alamofire/Alamofire.git", from: "5.8.0"),
        
        // WebSocket support
        .package(url: "https://github.com/daltoniam/Starscream.git", from: "4.0.0"),
        
        // JSON handling
        .package(url: "https://github.com/SwiftyJSON/SwiftyJSON.git", from: "5.0.0"),
        
        // Crypto utilities
        .package(url: "https://github.com/krzyzanowskim/CryptoSwift.git", from: "1.8.0"),
        
        // QR Code generation/scanning
        .package(url: "https://github.com/EFPrefix/EFQRCode.git", from: "6.2.0"),
        
        // Image loading and caching
        .package(url: "https://github.com/SDWebImage/SDWebImage.git", from: "5.18.0"),
        
        // Biometric authentication
        .package(url: "https://github.com/auth0/SimpleKeychain.git", from: "1.0.0"),
        
        // Analytics and crash reporting
        .package(url: "https://github.com/firebase/firebase-ios-sdk.git", from: "10.0.0")
    ],
    targets: [
        // MARK: - Main SDK Target
        .target(
            name: "FinovaSDK",
            dependencies: [
                "FinovaCore",
                "FinovaMining",
                "FinovaXP",
                "FinovaReferral",
                "FinovaNFT",
                "FinovaWallet",
                .product(name: "SolanaSwift", package: "solana-swift"),
                .product(name: "KeychainSwift", package: "keychain-swift"),
                .product(name: "Alamofire", package: "Alamofire"),
                .product(name: "Starscream", package: "Starscream"),
                .product(name: "SwiftyJSON", package: "SwiftyJSON"),
                .product(name: "CryptoSwift", package: "CryptoSwift"),
                .product(name: "EFQRCode", package: "EFQRCode"),
                .product(name: "SDWebImage", package: "SDWebImage"),
                .product(name: "SimpleKeychain", package: "SimpleKeychain"),
                .product(name: "FirebaseAnalytics", package: "firebase-ios-sdk"),
                .product(name: "FirebaseCrashlytics", package: "firebase-ios-sdk")
            ],
            path: "Sources/FinovaSDK",
            resources: [
                .process("Resources")
            ]
        ),
        
        // MARK: - Core Module
        .target(
            name: "FinovaCore",
            dependencies: [
                .product(name: "SolanaSwift", package: "solana-swift"),
                .product(name: "KeychainSwift", package: "keychain-swift"),
                .product(name: "Alamofire", package: "Alamofire"),
                .product(name: "SwiftyJSON", package: "SwiftyJSON"),
                .product(name: "CryptoSwift", package: "CryptoSwift")
            ],
            path: "Sources/FinovaCore",
            resources: [
                .process("Resources/Configs"),
                .process("Resources/Certificates")
            ]
        ),
        
        // MARK: - Mining Module
        .target(
            name: "FinovaMining",
            dependencies: [
                "FinovaCore",
                .product(name: "Starscream", package: "Starscream")
            ],
            path: "Sources/FinovaMining",
            resources: [
                .process("Resources/MiningAssets")
            ]
        ),
        
        // MARK: - XP System Module
        .target(
            name: "FinovaXP",
            dependencies: [
                "FinovaCore",
                .product(name: "SwiftyJSON", package: "SwiftyJSON")
            ],
            path: "Sources/FinovaXP",
            resources: [
                .process("Resources/XPAssets"),
                .process("Resources/BadgeIcons")
            ]
        ),
        
        // MARK: - Referral System Module
        .target(
            name: "FinovaReferral",
            dependencies: [
                "FinovaCore",
                .product(name: "EFQRCode", package: "EFQRCode")
            ],
            path: "Sources/FinovaReferral",
            resources: [
                .process("Resources/ReferralAssets")
            ]
        ),
        
        // MARK: - NFT Module
        .target(
            name: "FinovaNFT",
            dependencies: [
                "FinovaCore",
                .product(name: "SDWebImage", package: "SDWebImage"),
                .product(name: "SolanaSwift", package: "solana-swift")
            ],
            path: "Sources/FinovaNFT",
            resources: [
                .process("Resources/NFTAssets"),
                .process("Resources/CardTemplates")
            ]
        ),
        
        // MARK: - Wallet Module
        .target(
            name: "FinovaWallet",
            dependencies: [
                "FinovaCore",
                .product(name: "SimpleKeychain", package: "SimpleKeychain"),
                .product(name: "CryptoSwift", package: "CryptoSwift")
            ],
            path: "Sources/FinovaWallet",
            resources: [
                .process("Resources/WalletAssets"),
                .process("Resources/PaymentProviders")
            ]
        ),
        
        // MARK: - Test Targets
        .testTarget(
            name: "FinovaSDKTests",
            dependencies: [
                "FinovaSDK",
                "FinovaCore",
                "FinovaMining",
                "FinovaXP",
                "FinovaReferral",
                "FinovaNFT",
                "FinovaWallet"
            ],
            path: "Tests/FinovaSDKTests",
            resources: [
                .process("Resources/TestData"),
                .process("Resources/MockResponses")
            ]
        ),
        
        .testTarget(
            name: "FinovaCoreTests",
            dependencies: ["FinovaCore"],
            path: "Tests/FinovaCoreTests"
        ),
        
        .testTarget(
            name: "FinovaMiningTests",
            dependencies: ["FinovaMining", "FinovaCore"],
            path: "Tests/FinovaMiningTests"
        ),
        
        .testTarget(
            name: "FinovaXPTests",
            dependencies: ["FinovaXP", "FinovaCore"],
            path: "Tests/FinovaXPTests"
        ),
        
        .testTarget(
            name: "FinovaReferralTests",
            dependencies: ["FinovaReferral", "FinovaCore"],
            path: "Tests/FinovaReferralTests"
        ),
        
        .testTarget(
            name: "FinovaNFTTests",
            dependencies: ["FinovaNFT", "FinovaCore"],
            path: "Tests/FinovaNFTTests"
        ),
        
        .testTarget(
            name: "FinovaWalletTests",
            dependencies: ["FinovaWallet", "FinovaCore"],
            path: "Tests/FinovaWalletTests"
        )
    ],
    swiftLanguageVersions: [.v5]
)

// MARK: - Conditional Dependencies for Different Environments
#if os(iOS)
package.dependencies.append(
    .package(url: "https://github.com/auth0/Auth0.swift.git", from: "2.0.0")
)

// Add iOS-specific targets
package.targets.append(
    .target(
        name: "FinovaSocial",
        dependencies: [
            "FinovaCore",
            .product(name: "Auth0", package: "Auth0.swift")
        ],
        path: "Sources/FinovaSocial",
        resources: [
            .process("Resources/SocialPlatforms")
        ]
    )
)
#endif

// MARK: - Development Dependencies
#if DEBUG
package.dependencies.append(contentsOf: [
    .package(url: "https://github.com/pointfreeco/swift-snapshot-testing.git", from: "1.12.0"),
    .package(url: "https://github.com/Quick/Quick.git", from: "7.0.0"),
    .package(url: "https://github.com/Quick/Nimble.git", from: "12.0.0")
])
#endif
