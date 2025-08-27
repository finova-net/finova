Pod::Spec.new do |s|
  s.name             = 'FinovaReactNative'
  s.version          = '1.0.0'
  s.summary          = 'Finova Network React Native SDK - Social-Fi Super App with XP, RP & $FIN Mining'
  s.description      = <<-DESC
    Finova Network React Native SDK provides comprehensive integration for the next-generation Social-Fi Super App.
    Features include:
    - Integrated XP (Experience Points) system with gamified progression
    - RP (Referral Points) network building with exponential rewards
    - $FIN cryptocurrency mining with Pi Network-inspired algorithms
    - Multi-platform social media integration (Instagram, TikTok, YouTube, Facebook, X)
    - NFT marketplace with special cards and utility items
    - AI-powered content quality assessment
    - Anti-bot protection with Proof-of-Humanity
    - Liquid staking with auto-compounding rewards
    - Guild system and community governance
    - Real-time WebSocket connections for live updates
    - Indonesian e-wallet integration (OVO, GoPay, Dana, ShopeePay)
    - Comprehensive security with biometric authentication
  DESC
  
  s.homepage         = 'https://github.com/finova-network/finova-contracts'
  s.license          = { :type => 'MIT', :file => 'LICENSE' }
  s.author           = { 'Finova Network Team' => 'dev@finova.network' }
  s.source           = { 
    :git => 'https://github.com/finova-network/finova-contracts.git', 
    :tag => s.version.to_s 
  }
  
  # Platform Support
  s.ios.deployment_target = '12.0'
  s.android.deployment_target = '21'
  
  # React Native Compatibility
  s.requires_arc = true
  s.platforms = { :ios => "12.0" }
  
  # Source Files
  s.source_files = 'mobile-sdk/react-native/ios/**/*.{h,m,mm,swift}'
  s.public_header_files = 'mobile-sdk/react-native/ios/**/*.h'
  
  # Resource Files
  s.resources = [
    'mobile-sdk/react-native/assets/**/*',
    'config/ai-models/**/*.json',
    'config/blockchain/**/*.json'
  ]
  
  # Swift Version
  s.swift_version = '5.0'
  
  # React Native Dependency
  s.dependency 'React-Core'
  s.dependency 'React-RCTActionSheet'
  s.dependency 'React-RCTAlert'
  s.dependency 'React-RCTAnimation'
  s.dependency 'React-RCTBlob'
  s.dependency 'React-RCTImage'
  s.dependency 'React-RCTLinking'
  s.dependency 'React-RCTNetwork'
  s.dependency 'React-RCTSettings'
  s.dependency 'React-RCTText'
  s.dependency 'React-RCTVibration'
  s.dependency 'React-RCTWebSocket'
  
  # Core Dependencies for Finova Features
  s.dependency 'Alamofire', '~> 5.6'                    # Network requests
  s.dependency 'SwiftyJSON', '~> 5.0'                   # JSON parsing
  s.dependency 'KeychainAccess', '~> 4.2'               # Secure storage
  s.dependency 'CryptoSwift', '~> 1.6'                  # Cryptography
  s.dependency 'BigInt', '~> 5.3'                       # Large number handling
  
  # Blockchain & Web3 Dependencies
  s.dependency 'Web3Swift', '~> 3.0'                    # Ethereum compatibility
  s.dependency 'Solana.Swift', '~> 1.0'                 # Solana blockchain integration
  s.dependency 'WalletConnectSwift', '~> 1.7'           # Wallet connection
  s.dependency 'TweetNacl', '~> 1.1'                    # Ed25519 signatures
  
  # Biometric Authentication
  s.dependency 'LocalAuthentication'
  s.dependency 'BiometricAuthentication', '~> 3.2'      # Enhanced biometric support
  
  # Image Processing & Media
  s.dependency 'Kingfisher', '~> 7.0'                   # Image loading/caching
  s.dependency 'AVFoundation'                            # Video processing
  s.dependency 'Vision'                                  # AI image analysis
  
  # Social Media SDKs
  s.dependency 'FBSDKCoreKit', '~> 16.0'                 # Facebook integration
  s.dependency 'FBSDKLoginKit', '~> 16.0'                # Facebook login
  s.dependency 'GoogleSignIn', '~> 7.0'                 # Google/YouTube auth
  s.dependency 'TwitterKit', '~> 3.4'                   # X (Twitter) integration
  
  # Real-time Communication
  s.dependency 'SocketIO', '~> 4.7'                     # WebSocket connections
  s.dependency 'Starscream', '~> 4.0'                   # WebSocket client
  
  # Analytics & Monitoring
  s.dependency 'Firebase/Analytics'                      # User analytics
  s.dependency 'Firebase/Crashlytics'                    # Crash reporting
  s.dependency 'Firebase/Performance'                    # Performance monitoring
  s.dependency 'Firebase/RemoteConfig'                   # Feature flags
  
  # Payment Integration (Indonesian E-wallets)
  s.dependency 'Midtrans', '~> 1.17'                    # Indonesian payment gateway
  
  # UI/UX Enhancement
  s.dependency 'lottie-react-native', '~> 6.0'          # Animations
  s.dependency 'react-native-vector-icons', '~> 9.0'    # Icons
  s.dependency 'react-native-elements', '~> 3.4'        # UI components
  
  # Development Tools
  s.dependency 'React-RCTDevMenu' if ENV['RCT_DEV'] == '1'
  
  # Subspecs for Modular Integration
  s.subspec 'Core' do |core|
    core.source_files = 'mobile-sdk/react-native/ios/Core/**/*.{h,m,mm,swift}'
    core.public_header_files = 'mobile-sdk/react-native/ios/Core/**/*.h'
    core.dependency 'React-Core'
    core.dependency 'KeychainAccess'
    core.dependency 'CryptoSwift'
  end
  
  s.subspec 'Mining' do |mining|
    mining.source_files = 'mobile-sdk/react-native/ios/Mining/**/*.{h,m,mm,swift}'
    mining.public_header_files = 'mobile-sdk/react-native/ios/Mining/**/*.h'
    mining.dependency 'FinovaReactNative/Core'
    mining.dependency 'Solana.Swift'
    mining.dependency 'BigInt'
  end
  
  s.subspec 'Social' do |social|
    social.source_files = 'mobile-sdk/react-native/ios/Social/**/*.{h,m,mm,swift}'
    social.public_header_files = 'mobile-sdk/react-native/ios/Social/**/*.h'
    social.dependency 'FinovaReactNative/Core'
    social.dependency 'FBSDKCoreKit'
    social.dependency 'GoogleSignIn'
    social.dependency 'TwitterKit'
  end
  
  s.subspec 'XP' do |xp|
    xp.source_files = 'mobile-sdk/react-native/ios/XP/**/*.{h,m,mm,swift}'
    xp.public_header_files = 'mobile-sdk/react-native/ios/XP/**/*.h'
    xp.dependency 'FinovaReactNative/Core'
    xp.dependency 'Vision'
  end
  
  s.subspec 'RP' do |rp|
    rp.source_files = 'mobile-sdk/react-native/ios/RP/**/*.{h,m,mm,swift}'
    rp.public_header_files = 'mobile-sdk/react-native/ios/RP/**/*.h'
    rp.dependency 'FinovaReactNative/Core'
    rp.dependency 'FinovaReactNative/Social'
  end
  
  s.subspec 'NFT' do |nft|
    nft.source_files = 'mobile-sdk/react-native/ios/NFT/**/*.{h,m,mm,swift}'
    nft.public_header_files = 'mobile-sdk/react-native/ios/NFT/**/*.h'
    nft.dependency 'FinovaReactNative/Core'
    nft.dependency 'Kingfisher'
    nft.dependency 'Web3Swift'
  end
  
  s.subspec 'Staking' do |staking|
    staking.source_files = 'mobile-sdk/react-native/ios/Staking/**/*.{h,m,mm,swift}'
    staking.public_header_files = 'mobile-sdk/react-native/ios/Staking/**/*.h'
    staking.dependency 'FinovaReactNative/Core'
    staking.dependency 'FinovaReactNative/Mining'
    staking.dependency 'BigInt'
  end
  
  s.subspec 'Wallet' do |wallet|
    wallet.source_files = 'mobile-sdk/react-native/ios/Wallet/**/*.{h,m,mm,swift}'
    wallet.public_header_files = 'mobile-sdk/react-native/ios/Wallet/**/*.h'
    wallet.dependency 'FinovaReactNative/Core'
    wallet.dependency 'WalletConnectSwift'
    wallet.dependency 'BiometricAuthentication'
    wallet.dependency 'Midtrans'
  end
  
  s.subspec 'AI' do |ai|
    ai.source_files = 'mobile-sdk/react-native/ios/AI/**/*.{h,m,mm,swift}'
    ai.public_header_files = 'mobile-sdk/react-native/ios/AI/**/*.h'
    ai.dependency 'FinovaReactNative/Core'
    ai.dependency 'Vision'
    ai.dependency 'CoreML'
  end
  
  s.subspec 'Guild' do |guild|
    guild.source_files = 'mobile-sdk/react-native/ios/Guild/**/*.{h,m,mm,swift}'
    guild.public_header_files = 'mobile-sdk/react-native/ios/Guild/**/*.h'
    guild.dependency 'FinovaReactNative/Core'
    guild.dependency 'SocketIO'
  end
  
  s.subspec 'Analytics' do |analytics|
    analytics.source_files = 'mobile-sdk/react-native/ios/Analytics/**/*.{h,m,mm,swift}'
    analytics.public_header_files = 'mobile-sdk/react-native/ios/Analytics/**/*.h'
    analytics.dependency 'FinovaReactNative/Core'
    analytics.dependency 'Firebase/Analytics'
    analytics.dependency 'Firebase/Performance'
  end
  
  # Framework Search Paths
  s.framework = 'Foundation', 'UIKit', 'Security', 'LocalAuthentication', 'Vision', 'AVFoundation', 'CoreML'
  s.library = 'c++'
  
  # Header Search Paths
  s.xcconfig = {
    'HEADER_SEARCH_PATHS' => '$(PODS_ROOT)/Headers/Public/React-Core/**',
    'OTHER_LDFLAGS' => '-ObjC',
    'ENABLE_BITCODE' => 'NO',
    'SWIFT_VERSION' => '5.0',
    'DEFINES_MODULE' => 'YES'
  }
  
  # Build Settings
  s.pod_target_xcconfig = {
    'GCC_PREPROCESSOR_DEFINITIONS' => '$(inherited) COCOAPODS=1 FINOVA_SDK=1',
    'CLANG_ENABLE_MODULES' => 'YES',
    'SWIFT_COMPILATION_MODE' => 'wholemodule',
    'SWIFT_OPTIMIZATION_LEVEL' => '-O'
  }
  
  # Resource Bundle
  s.resource_bundles = {
    'FinovaReactNative' => [
      'mobile-sdk/react-native/assets/**/*.{png,jpg,gif,json,ttf,otf}',
      'config/ai-models/**/*.{json,mlmodel}',
      'config/blockchain/program-addresses.json'
    ]
  }
  
  # Script Phases for Code Generation
  s.script_phase = {
    :name => 'Generate Finova Types',
    :script => <<-SCRIPT
      echo "Generating Finova SDK types from smart contracts..."
      if [ -f "${PODS_TARGET_SRCROOT}/tools/code-generation/client-generator/typescript/generator.js" ]; then
        node "${PODS_TARGET_SRCROOT}/tools/code-generation/client-generator/typescript/generator.js" \
          --input "${PODS_TARGET_SRCROOT}/programs" \
          --output "${PODS_TARGET_SRCROOT}/mobile-sdk/react-native/src/types" \
          --platform "react-native"
        echo "✅ Type generation completed"
      else
        echo "⚠️  Type generator not found, using pre-generated types"
      fi
    SCRIPT,
    :execution_position => :before_compile
  }
  
  # Prepare Command for Development
  s.prepare_command = <<-CMD
    echo "🚀 Preparing Finova React Native SDK..."
    
    # Create necessary directories
    mkdir -p mobile-sdk/react-native/ios/{Core,Mining,Social,XP,RP,NFT,Staking,Wallet,AI,Guild,Analytics}
    
    # Copy configuration files
    if [ ! -f "mobile-sdk/react-native/finova.config.json" ]; then
      cp config/environments/development.json mobile-sdk/react-native/finova.config.json
      echo "📄 Configuration file created"
    fi
    
    # Generate TypeScript definitions if tools are available
    if [ -d "tools/code-generation" ]; then
      echo "🔧 Generating TypeScript definitions..."
    fi
    
    echo "✅ Finova SDK preparation completed"
  CMD
  
  # Validation
  s.validate_spec = true
  
  # Documentation
  s.documentation_url = 'https://docs.finova.network/sdk/react-native'
  
  # Preserve Path for React Native Metro
  s.preserve_paths = [
    'mobile-sdk/react-native/package.json',
    'mobile-sdk/react-native/src/**/*',
    'mobile-sdk/react-native/assets/**/*'
  ]
  
  # Test Spec
  s.test_spec 'Tests' do |test_spec|
    test_spec.source_files = 'tests/unit/mobile-sdk/react-native/**/*.{m,mm,swift}'
    test_spec.dependency 'Quick', '~> 6.0'
    test_spec.dependency 'Nimble', '~> 11.0'
  end
  
  # App Extension API Only (for widgets/extensions)
  s.app_spec 'FinovaWidget' do |app_spec|
    app_spec.source_files = 'mobile-sdk/react-native/ios/Widget/**/*.{h,m,mm,swift}'
    app_spec.dependency 'FinovaReactNative/Core'
  end
  
  # Compiler Flags
  s.compiler_flags = '-DFINOVA_SDK_VERSION=\"1.0.0\"'
  
  # Module Map
  s.module_map = 'mobile-sdk/react-native/ios/FinovaReactNative.modulemap'
  
  # Info.plist Configuration
  s.info_plist = {
    'CFBundleIdentifier' => 'network.finova.react-native-sdk',
    'CFBundleVersion' => s.version.to_s,
    'CFBundleShortVersionString' => s.version.to_s,
    'NSCameraUsageDescription' => 'Finova uses camera for KYC verification and content creation',
    'NSMicrophoneUsageDescription' => 'Finova uses microphone for video content creation',
    'NSPhotoLibraryUsageDescription' => 'Finova accesses photo library for content sharing and profile customization',
    'NSFaceIDUsageDescription' => 'Finova uses Face ID for secure authentication and wallet access',
    'NSContactsUsageDescription' => 'Finova accesses contacts for referral system (optional)',
    'NSLocationWhenInUseUsageDescription' => 'Finova uses location for regional features and anti-fraud protection'
  }
  
  # Minimum iOS Version Check
  s.ios.deployment_target = '12.0'
  
  # Exclude Architecture for Simulator (if needed for specific dependencies)
  s.pod_target_xcconfig = {
    'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'arm64'
  } if ENV['SIMULATOR_EXCLUDED_ARCHS'] == '1'
  
  # License Acknowledgment
  s.license = {
    :type => 'MIT',
    :text => <<-LICENSE
      MIT License
      
      Copyright (c) 2025 Finova Network
      
      Permission is hereby granted, free of charge, to any person obtaining a copy
      of this software and associated documentation files (the "Software"), to deal
      in the Software without restriction, including without limitation the rights
      to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
      copies of the Software, and to permit persons to whom the Software is
      furnished to do so, subject to the following conditions:
      
      The above copyright notice and this permission notice shall be included in all
      copies or substantial portions of the Software.
      
      THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
      IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
      FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
      AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
      LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
      OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
      SOFTWARE.
    LICENSE
  }
  
  # Changelog URL
  s.changelog = 'https://github.com/finova-network/finova-contracts/blob/main/CHANGELOG.md'
  
  # Social Media Links
  s.social_media_url = 'https://twitter.com/FinovaNetwork'
  
  # Screenshots for documentation
  s.screenshots = [
    'https://github.com/finova-network/finova-contracts/raw/main/docs/assets/screenshots/mining-interface.png',
    'https://github.com/finova-network/finova-contracts/raw/main/docs/assets/screenshots/xp-dashboard.png',
    'https://github.com/finova-network/finova-contracts/raw/main/docs/assets/screenshots/referral-network.png'
  ]
end
