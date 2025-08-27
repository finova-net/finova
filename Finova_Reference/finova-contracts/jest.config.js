/**
 * Finova Network - Enterprise-Grade Jest Configuration
 * 
 * Comprehensive testing configuration for:
 * - Smart Contracts (Anchor/Solana)
 * - TypeScript/JavaScript APIs
 * - Client SDKs (TS, React Native)
 * - Integration Tests
 * - Security Tests
 * - Performance Tests
 * - Mobile App Testing
 * 
 * @version 1.0.0
 * @author Finova Network Development Team
 * @license MIT
 */

const path = require('path');
const { pathsToModuleNameMap } = require('ts-jest');
const { compilerOptions } = require('./tsconfig.json');

// Environment detection
const isCI = process.env.CI === 'true';
const isDocker = process.env.DOCKER_ENV === 'true';
const testEnv = process.env.TEST_ENV || 'development';
const enableCoverage = process.env.ENABLE_COVERAGE !== 'false';

// Performance settings based on environment
const performanceConfig = {
  maxWorkers: isCI ? 2 : '50%',
  workerIdleMemoryLimit: isCI ? '512MB' : '1GB',
  detectOpenHandles: !isCI,
  forceExit: isCI,
  verbose: !isCI
};

// Base configuration
const baseConfig = {
  // Core Jest Settings
  displayName: {
    name: 'FINOVA-NETWORK',
    color: 'cyan'
  },
  
  // Test Environment
  testEnvironment: 'node',
  
  // Performance Configuration
  ...performanceConfig,
  
  // Timeout Settings
  testTimeout: 30000, // 30 seconds for blockchain operations
  setupFilesAfterEnv: [
    '<rootDir>/tests/config/setup-tests.ts',
    '<rootDir>/tests/helpers/setup.ts'
  ],
  
  // Global Setup and Teardown
  globalSetup: '<rootDir>/tests/config/global-setup.ts',
  globalTeardown: '<rootDir>/tests/config/global-teardown.ts',
  
  // Module Resolution
  moduleNameMapper: {
    // TypeScript path mappings
    ...pathsToModuleNameMap(compilerOptions.paths || {}, {
      prefix: '<rootDir>/'
    }),
    
    // Anchor/Solana mappings
    '^@finova/core$': '<rootDir>/programs/finova-core/src',
    '^@finova/token$': '<rootDir>/programs/finova-token/src',
    '^@finova/nft$': '<rootDir>/programs/finova-nft/src',
    '^@finova/defi$': '<rootDir>/programs/finova-defi/src',
    '^@finova/oracle$': '<rootDir>/programs/finova-oracle/src',
    '^@finova/bridge$': '<rootDir>/programs/finova-bridge/src',
    
    // Client SDK mappings
    '^@finova/sdk-typescript$': '<rootDir>/client/typescript/src',
    '^@finova/sdk-rust$': '<rootDir>/client/rust/src',
    '^@finova/sdk-python$': '<rootDir>/client/python/finova',
    
    // Mobile SDK mappings
    '^@finova/mobile-ios$': '<rootDir>/mobile-sdk/ios/FinovaSDK/Sources',
    '^@finova/mobile-android$': '<rootDir>/mobile-sdk/android/finova-sdk/src/main/java',
    '^@finova/mobile-rn$': '<rootDir>/mobile-sdk/react-native/src',
    
    // API mappings
    '^@finova/api$': '<rootDir>/api/src',
    '^@finova/api/(.*)$': '<rootDir>/api/src/$1',
    
    // AI Services mappings
    '^@finova/ai-content$': '<rootDir>/ai-services/content-analyzer/src',
    '^@finova/ai-bot$': '<rootDir>/ai-services/bot-detection/src',
    '^@finova/ai-recommendation$': '<rootDir>/ai-services/recommendation/src',
    '^@finova/ai-analytics$': '<rootDir>/ai-services/analytics/src',
    
    // Utilities and helpers
    '^@finova/utils$': '<rootDir>/tests/utils',
    '^@finova/fixtures$': '<rootDir>/tests/fixtures',
    '^@finova/mocks$': '<rootDir>/tests/helpers/mocks',
    
    // Static asset mocking
    '\\.(css|less|scss|sass)$': 'identity-obj-proxy',
    '\\.(jpg|jpeg|png|gif|svg)$': '<rootDir>/tests/helpers/__mocks__/file-mock.js'
  },
  
  // File Extensions
  moduleFileExtensions: [
    'ts',
    'tsx',
    'js',
    'jsx',
    'json',
    'node',
    'rs', // Rust files
    'py', // Python files
    'swift', // iOS Swift files
    'kt', // Kotlin files for Android
    'java' // Android Java files
  ],
  
  // Transform Configuration
  transform: {
    // TypeScript/JavaScript transformation
    '^.+\\.(ts|tsx)$': ['ts-jest', {
      tsconfig: './tsconfig.json',
      isolatedModules: true,
      diagnostics: {
        ignoreCodes: [1343]
      },
      astTransformers: {
        before: [
          {
            path: '<rootDir>/tests/transformers/anchor-transformer.ts',
            options: {}
          }
        ]
      }
    }],
    
    // JavaScript ES6+ transformation
    '^.+\\.(js|jsx)$': ['babel-jest', {
      presets: [
        ['@babel/preset-env', { targets: { node: 'current' } }],
        '@babel/preset-typescript',
        '@babel/preset-react'
      ],
      plugins: [
        '@babel/plugin-proposal-class-properties',
        '@babel/plugin-proposal-private-methods',
        '@babel/plugin-proposal-optional-chaining',
        '@babel/plugin-proposal-nullish-coalescing-operator'
      ]
    }],
    
    // Rust file handling (for documentation/analysis)
    '^.+\\.rs$': '<rootDir>/tests/transformers/rust-transformer.js',
    
    // Python file handling
    '^.+\\.py$': '<rootDir>/tests/transformers/python-transformer.js',
    
    // Mobile file handling
    '^.+\\.(swift|kt|java)$': '<rootDir>/tests/transformers/mobile-transformer.js'
  },
  
  // Transform Ignore Patterns
  transformIgnorePatterns: [
    'node_modules/(?!(.*\\.mjs$|@solana|@project-serum|@coral-xyz|buffer|bn.js))',
    '\\.pnp\\.[^\\\/]+$'
  ],
  
  // Test Match Patterns
  testMatch: [
    '<rootDir>/tests/**/*.(test|spec).(ts|tsx|js|jsx)',
    '<rootDir>/programs/**/tests/**/*.(test|spec).(ts|tsx|js)',
    '<rootDir>/client/**/tests/**/*.(test|spec).(ts|tsx|js)',
    '<rootDir>/api/**/tests/**/*.(test|spec).(ts|tsx|js)',
    '<rootDir>/mobile-sdk/**/tests/**/*.(test|spec).(ts|tsx|js)',
    '<rootDir>/ai-services/**/tests/**/*.(test|spec).(ts|tsx|js|py)'
  ],
  
  // Test Path Ignore Patterns
  testPathIgnorePatterns: [
    '<rootDir>/node_modules/',
    '<rootDir>/dist/',
    '<rootDir>/build/',
    '<rootDir>/target/',
    '<rootDir>/.anchor/',
    '<rootDir>/coverage/',
    '<rootDir>/docs/',
    '<rootDir>/infrastructure/',
    '<rootDir>/scripts/(?!.*test)'
  ],
  
  // Coverage Configuration
  collectCoverage: enableCoverage,
  collectCoverageFrom: [
    // Smart Contract Coverage
    'programs/*/src/**/*.{ts,js}',
    '!programs/*/src/**/*.d.ts',
    '!programs/*/src/**/index.{ts,js}',
    
    // Client SDK Coverage
    'client/*/src/**/*.{ts,tsx,js,jsx}',
    '!client/*/src/**/*.d.ts',
    '!client/*/src/**/index.{ts,tsx,js,jsx}',
    
    // API Coverage
    'api/src/**/*.{ts,js}',
    '!api/src/**/*.d.ts',
    '!api/src/**/index.{ts,js}',
    '!api/src/**/*.config.{ts,js}',
    
    // Mobile SDK Coverage
    'mobile-sdk/*/src/**/*.{ts,tsx,js,jsx}',
    '!mobile-sdk/*/src/**/*.d.ts',
    
    // AI Services Coverage
    'ai-services/*/src/**/*.{ts,js,py}',
    '!ai-services/*/src/**/*.d.ts',
    
    // Exclude patterns
    '!**/__tests__/**',
    '!**/__mocks__/**',
    '!**/tests/**',
    '!**/coverage/**',
    '!**/node_modules/**',
    '!**/dist/**',
    '!**/build/**'
  ],
  
  // Coverage Reporters
  coverageReporters: [
    'text',
    'text-summary',
    'html',
    'lcov',
    'json',
    'clover',
    ...(isCI ? ['cobertura'] : [])
  ],
  
  // Coverage Directory
  coverageDirectory: '<rootDir>/coverage',
  
  // Coverage Thresholds
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 85,
      lines: 85,
      statements: 85
    },
    // Smart Contract specific thresholds (higher requirements)
    './programs/finova-core/src/**/*.{ts,js}': {
      branches: 90,
      functions: 95,
      lines: 95,
      statements: 95
    },
    // API specific thresholds
    './api/src/**/*.{ts,js}': {
      branches: 85,
      functions: 90,
      lines: 90,
      statements: 90
    }
  },
  
  // Reporter Configuration
  reporters: [
    'default',
    ['jest-html-reporters', {
      publicPath: './coverage/html-report',
      filename: 'report.html',
      expand: true,
      hideIcon: false,
      pageTitle: 'Finova Network Test Report'
    }],
    ['jest-junit', {
      outputDirectory: './coverage',
      outputName: 'junit.xml',
      ancestorSeparator: ' › ',
      uniqueOutputName: 'false',
      suiteNameTemplate: '{filepath}',
      classNameTemplate: '{classname}',
      titleTemplate: '{title}'
    }],
    ...(isCI ? [
      ['github-actions', {
        silent: false
      }]
    ] : [])
  ],
  
  // Cache Configuration
  cacheDirectory: '<rootDir>/.jest-cache',
  clearMocks: true,
  restoreMocks: true,
  resetMocks: true,
  
  // Watch Configuration
  watchPathIgnorePatterns: [
    '<rootDir>/node_modules/',
    '<rootDir>/coverage/',
    '<rootDir>/dist/',
    '<rootDir>/build/',
    '<rootDir>/target/',
    '<rootDir>/.anchor/'
  ],
  
  // Error Handling
  bail: isCI ? 1 : 0,
  errorOnDeprecated: true,
  testFailureExitCode: 1,
  
  // Advanced Configuration
  extensionsToTreatAsEsm: ['.ts', '.tsx'],
  
  // Global Variables
  globals: {
    'ts-jest': {
      useESM: true,
      isolatedModules: true
    },
    // Solana/Anchor globals
    __SOLANA_NETWORK__: testEnv,
    __ANCHOR_PROVIDER_URL__: process.env.ANCHOR_PROVIDER_URL || 'http://127.0.0.1:8899',
    __PROGRAM_ID_CORE__: process.env.PROGRAM_ID_CORE || '11111111111111111111111111111111',
    __PROGRAM_ID_TOKEN__: process.env.PROGRAM_ID_TOKEN || '11111111111111111111111111111112',
    __PROGRAM_ID_NFT__: process.env.PROGRAM_ID_NFT || '11111111111111111111111111111113',
    // API globals
    __API_BASE_URL__: process.env.API_BASE_URL || 'http://localhost:3000',
    __DATABASE_URL__: process.env.DATABASE_URL || 'postgresql://localhost:5432/finova_test',
    __REDIS_URL__: process.env.REDIS_URL || 'redis://localhost:6379',
    // Mobile testing globals
    __MOBILE_TEST_MODE__: process.env.MOBILE_TEST_MODE || 'simulator',
    // AI services globals
    __AI_MODEL_PATH__: process.env.AI_MODEL_PATH || './tests/fixtures/models',
    __ENABLE_AI_TESTS__: process.env.ENABLE_AI_TESTS === 'true'
  }
};

// Project-specific configurations
const projects = [
  // Smart Contracts Testing
  {
    ...baseConfig,
    displayName: {
      name: 'SMART-CONTRACTS',
      color: 'yellow'
    },
    testMatch: [
      '<rootDir>/programs/**/tests/**/*.(test|spec).(ts|js)',
      '<rootDir>/tests/unit/programs/**/*.(test|spec).(ts|js)',
      '<rootDir>/tests/integration/cross-program/**/*.(test|spec).(ts|js)'
    ],
    setupFilesAfterEnv: [
      '<rootDir>/tests/config/setup-tests.ts',
      '<rootDir>/tests/config/setup-anchor.ts'
    ],
    testEnvironment: '<rootDir>/tests/environments/anchor-environment.js',
    globals: {
      ...baseConfig.globals,
      __TEST_TYPE__: 'smart-contracts'
    },
    testTimeout: 60000 // Extended timeout for blockchain operations
  },
  
  // API Testing
  {
    ...baseConfig,
    displayName: {
      name: 'API',
      color: 'green'
    },
    testMatch: [
      '<rootDir>/api/**/*.(test|spec).(ts|js)',
      '<rootDir>/tests/unit/api/**/*.(test|spec).(ts|js)',
      '<rootDir>/tests/integration/api-blockchain/**/*.(test|spec).(ts|js)'
    ],
    setupFilesAfterEnv: [
      '<rootDir>/tests/config/setup-tests.ts',
      '<rootDir>/tests/config/setup-api.ts'
    ],
    testEnvironment: '<rootDir>/tests/environments/api-environment.js',
    globals: {
      ...baseConfig.globals,
      __TEST_TYPE__: 'api'
    }
  },
  
  // Client SDK Testing
  {
    ...baseConfig,
    displayName: {
      name: 'CLIENT-SDKs',
      color: 'blue'
    },
    testMatch: [
      '<rootDir>/client/**/*.(test|spec).(ts|tsx|js|jsx)',
      '<rootDir>/tests/unit/client/**/*.(test|spec).(ts|tsx|js|jsx)'
    ],
    setupFilesAfterEnv: [
      '<rootDir>/tests/config/setup-tests.ts',
      '<rootDir>/tests/config/setup-client.ts'
    ],
    globals: {
      ...baseConfig.globals,
      __TEST_TYPE__: 'client-sdk'
    }
  },
  
  // Mobile SDK Testing
  {
    ...baseConfig,
    displayName: {
      name: 'MOBILE-SDKs',
      color: 'magenta'
    },
    testMatch: [
      '<rootDir>/mobile-sdk/**/*.(test|spec).(ts|tsx|js|jsx)',
      '<rootDir>/tests/unit/mobile/**/*.(test|spec).(ts|tsx|js|jsx)'
    ],
    setupFilesAfterEnv: [
      '<rootDir>/tests/config/setup-tests.ts',
      '<rootDir>/tests/config/setup-mobile.ts'
    ],
    testEnvironment: '<rootDir>/tests/environments/mobile-environment.js',
    globals: {
      ...baseConfig.globals,
      __TEST_TYPE__: 'mobile-sdk',
      __REACT_NATIVE_VERSION__: process.env.REACT_NATIVE_VERSION || '0.72.0'
    }
  },
  
  // AI Services Testing
  {
    ...baseConfig,
    displayName: {
      name: 'AI-SERVICES',
      color: 'cyan'
    },
    testMatch: [
      '<rootDir>/ai-services/**/*.(test|spec).(ts|js|py)',
      '<rootDir>/tests/unit/ai/**/*.(test|spec).(ts|js)'
    ],
    setupFilesAfterEnv: [
      '<rootDir>/tests/config/setup-tests.ts',
      '<rootDir>/tests/config/setup-ai.ts'
    ],
    testEnvironment: '<rootDir>/tests/environments/ai-environment.js',
    globals: {
      ...baseConfig.globals,
      __TEST_TYPE__: 'ai-services'
    },
    testTimeout: 120000 // Extended timeout for AI model operations
  },
  
  // Integration Testing
  {
    ...baseConfig,
    displayName: {
      name: 'INTEGRATION',
      color: 'red'
    },
    testMatch: [
      '<rootDir>/tests/integration/**/*.(test|spec).(ts|js)',
      '<rootDir>/tests/e2e/**/*.(test|spec).(ts|js)'
    ],
    setupFilesAfterEnv: [
      '<rootDir>/tests/config/setup-tests.ts',
      '<rootDir>/tests/config/setup-integration.ts'
    ],
    testEnvironment: '<rootDir>/tests/environments/integration-environment.js',
    globals: {
      ...baseConfig.globals,
      __TEST_TYPE__: 'integration'
    },
    testTimeout: 300000, // 5 minutes for complex integration tests
    maxWorkers: 1 // Sequential execution for integration tests
  },
  
  // Security Testing
  {
    ...baseConfig,
    displayName: {
      name: 'SECURITY',
      color: 'red'
    },
    testMatch: [
      '<rootDir>/tests/security/**/*.(test|spec).(ts|js)'
    ],
    setupFilesAfterEnv: [
      '<rootDir>/tests/config/setup-tests.ts',
      '<rootDir>/tests/config/setup-security.ts'
    ],
    testEnvironment: '<rootDir>/tests/environments/security-environment.js',
    globals: {
      ...baseConfig.globals,
      __TEST_TYPE__: 'security',
      __SECURITY_TEST_MODE__: 'comprehensive'
    },
    testTimeout: 180000 // 3 minutes for security tests
  },
  
  // Performance Testing
  {
    ...baseConfig,
    displayName: {
      name: 'PERFORMANCE',
      color: 'yellow'
    },
    testMatch: [
      '<rootDir>/tests/load/**/*.(test|spec).(ts|js)'
    ],
    setupFilesAfterEnv: [
      '<rootDir>/tests/config/setup-tests.ts',
      '<rootDir>/tests/config/setup-performance.ts'
    ],
    testEnvironment: '<rootDir>/tests/environments/performance-environment.js',
    globals: {
      ...baseConfig.globals,
      __TEST_TYPE__: 'performance'
    },
    testTimeout: 600000, // 10 minutes for performance tests
    maxWorkers: 1 // Single worker for accurate performance measurement
  }
];

// Export configuration based on test type
const testType = process.env.TEST_TYPE;

if (testType && testType !== 'all') {
  // Export specific project configuration
  const project = projects.find(p => 
    p.displayName.name.toLowerCase().includes(testType.toLowerCase())
  );
  
  if (project) {
    module.exports = project;
  } else {
    console.warn(`Test type "${testType}" not found. Using base configuration.`);
    module.exports = baseConfig;
  }
} else {
  // Export multi-project configuration
  module.exports = {
    ...baseConfig,
    projects: projects,
    
    // Multi-project specific settings
    collectCoverageFrom: [
      ...baseConfig.collectCoverageFrom,
      // Additional patterns for multi-project coverage
      'tools/**/*.{ts,js}',
      '!tools/**/node_modules/**'
    ],
    
    // Watch mode configuration for multi-project
    watchPlugins: [
      'jest-watch-typeahead/filename',
      'jest-watch-typeahead/testname',
      'jest-watch-select-projects'
    ],
    
    // Notification configuration
    notify: !isCI,
    notifyMode: 'failure-change',
    
    // Test sequencing
    testSequencer: '<rootDir>/tests/config/test-sequencer.js',
    
    // Custom resolver for complex module resolution
    resolver: '<rootDir>/tests/config/custom-resolver.js',
    
    // Snapshot configuration
    snapshotSerializers: [
      '<rootDir>/tests/config/blockchain-snapshot-serializer.js',
      '<rootDir>/tests/config/bigint-snapshot-serializer.js'
    ]
  };
}

// Development mode helpers
if (process.env.NODE_ENV === 'development') {
  // Add development-specific configurations
  module.exports.watchman = true;
  module.exports.watch = process.env.JEST_WATCH === 'true';
  module.exports.coverage = false; // Disable coverage in development watch mode
}

// CI/CD optimizations
if (isCI) {
  // Optimize for CI environment
  module.exports.ci = true;
  module.exports.silent = false;
  module.exports.verbose = false;
  module.exports.passWithNoTests = true;
  
  // Enhanced error reporting in CI
  module.exports.reporters.push([
    'jest-silent-reporter',
    {
      useDots: true,
      showWarnings: true
    }
  ]);
}

// Docker environment optimizations
if (isDocker) {
  // Optimize for Docker environment
  module.exports.maxWorkers = 2;
  module.exports.workerIdleMemoryLimit = '256MB';
  module.exports.cache = false; // Disable cache in Docker
}

/**
 * Custom Jest Extensions and Utilities
 */

// Add custom matchers path
if (baseConfig.setupFilesAfterEnv) {
  baseConfig.setupFilesAfterEnv.push('<rootDir>/tests/config/custom-matchers.ts');
}

// Environment-specific test file patterns
const environmentTestPatterns = {
  development: ['**/*.dev.(test|spec).(ts|js)'],
  staging: ['**/*.staging.(test|spec).(ts|js)'],
  production: ['**/*.prod.(test|spec).(ts|js)']
};

if (environmentTestPatterns[testEnv]) {
  if (Array.isArray(module.exports.testMatch)) {
    module.exports.testMatch.push(...environmentTestPatterns[testEnv]);
  } else if (module.exports.projects) {
    module.exports.projects.forEach(project => {
      if (project.testMatch) {
        project.testMatch.push(...environmentTestPatterns[testEnv]);
      }
    });
  }
}

/**
 * Validation and Error Handling
 */
const validateConfig = (config) => {
  const errors = [];
  
  if (!config.testMatch || config.testMatch.length === 0) {
    errors.push('testMatch is required and must contain at least one pattern');
  }
  
  if (config.collectCoverage && (!config.collectCoverageFrom || config.collectCoverageFrom.length === 0)) {
    errors.push('collectCoverageFrom is required when collectCoverage is true');
  }
  
  if (errors.length > 0) {
    throw new Error(`Jest configuration validation failed:\n${errors.join('\n')}`);
  }
  
  return true;
};

// Perform validation
try {
  validateConfig(module.exports);
} catch (error) {
  console.error('Jest Configuration Error:', error.message);
  process.exit(1);
}

// Export configuration metadata for tooling
module.exports.__meta = {
  version: '1.0.0',
  supportedNodeVersions: ['>=16.0.0'],
  requiredDependencies: [
    'jest',
    'ts-jest',
    '@types/jest',
    'babel-jest',
    '@babel/core',
    '@babel/preset-env',
    '@babel/preset-typescript'
  ],
  optionalDependencies: [
    'jest-html-reporters',
    'jest-junit',
    '@jest/test-sequencer',
    'jest-watch-typeahead'
  ],
  environments: Object.keys(environmentTestPatterns),
  testTypes: projects.map(p => p.displayName.name.toLowerCase()),
  features: [
    'multi-project-support',
    'smart-contract-testing',
    'mobile-sdk-testing',
    'ai-services-testing',
    'security-testing',
    'performance-testing',
    'code-coverage',
    'ci-cd-integration',
    'docker-support',
    'custom-matchers',
    'snapshot-testing',
    'parallel-execution'
  ]
};

console.log(`
🚀 Finova Network Jest Configuration Loaded
📊 Test Projects: ${projects.length}
🔧 Environment: ${testEnv}
📈 Coverage: ${enableCoverage ? 'Enabled' : 'Disabled'}
⚡ Max Workers: ${performanceConfig.maxWorkers}
🐳 Docker Mode: ${isDocker ? 'Yes' : 'No'}
🤖 CI Mode: ${isCI ? 'Yes' : 'No'}
`);
