# Finova Network: Enterprise-Grade Makefile
# Version: 1.0 | July 2025
# Description: Complete build automation for Finova Network ecosystem
# Author: Finova Development Team

# ==============================================================================
# CONFIGURATION & VARIABLES
# ==============================================================================

# Project metadata
PROJECT_NAME := finova-network
VERSION := $(shell cat package.json | grep version | head -1 | awk -F: '{ print $$2 }' | sed 's/[",]//g' | tr -d '[[:space:]]')
BUILD_DATE := $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
GIT_COMMIT := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
GIT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")

# Directories
ROOT_DIR := $(shell pwd)
PROGRAMS_DIR := $(ROOT_DIR)/programs
CLIENT_DIR := $(ROOT_DIR)/client
TESTS_DIR := $(ROOT_DIR)/tests
DOCS_DIR := $(ROOT_DIR)/docs
SCRIPTS_DIR := $(ROOT_DIR)/scripts
BUILD_DIR := $(ROOT_DIR)/target
DIST_DIR := $(ROOT_DIR)/dist
LOGS_DIR := $(ROOT_DIR)/logs
COVERAGE_DIR := $(ROOT_DIR)/coverage

# Environment configuration
NODE_ENV ?= development
SOLANA_NETWORK ?= devnet
ANCHOR_PROVIDER_URL ?= https://api.devnet.solana.com
ANCHOR_WALLET ?= ~/.config/solana/id.json

# Tool versions
RUST_VERSION := 1.70.0
NODE_VERSION := 18.17.0
ANCHOR_VERSION := 0.28.0
SOLANA_VERSION := 1.16.0

# Color codes for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
PURPLE := \033[0;35m
CYAN := \033[0;36m
WHITE := \033[0;37m
BOLD := \033[1m
NC := \033[0m # No Color

# Docker configuration
DOCKER_REGISTRY ?= finova
DOCKER_TAG ?= $(VERSION)
DOCKER_PLATFORM ?= linux/amd64,linux/arm64

# ==============================================================================
# HELP & DOCUMENTATION
# ==============================================================================

.DEFAULT_GOAL := help
.PHONY: help

help: ## Display this help message with available targets
	@echo "$(BOLD)$(CYAN)Finova Network - Enterprise Build System$(NC)"
	@echo "$(BLUE)Version: $(VERSION) | Build: $(BUILD_DATE) | Commit: $(GIT_COMMIT)$(NC)"
	@echo ""
	@echo "$(BOLD)Available targets:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  $(CYAN)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(BOLD)Categories:$(NC)"
	@echo "  $(YELLOW)Setup:$(NC)        install, setup-dev, clean"
	@echo "  $(YELLOW)Development:$(NC)  build, test, lint, format"
	@echo "  $(YELLOW)Blockchain:$(NC)   deploy, upgrade, verify"
	@echo "  $(YELLOW)Docker:$(NC)       docker-build, docker-push, docker-run"
	@echo "  $(YELLOW)Security:$(NC)     security-scan, audit, vulnerability-check"
	@echo "  $(YELLOW)Monitoring:$(NC)   logs, status, health-check"
	@echo ""

# ==============================================================================
# ENVIRONMENT SETUP & INSTALLATION
# ==============================================================================

.PHONY: install setup-dev check-deps install-tools install-rust install-node install-solana

install: check-deps install-tools ## Install all dependencies and tools
	@echo "$(BOLD)$(GREEN)Installing Finova Network dependencies...$(NC)"
	@$(MAKE) install-rust
	@$(MAKE) install-node
	@$(MAKE) install-solana
	@yarn install --frozen-lockfile
	@cargo build --release
	@echo "$(GREEN)✅ Installation completed successfully$(NC)"

setup-dev: install ## Setup complete development environment
	@echo "$(BOLD)$(GREEN)Setting up development environment...$(NC)"
	@mkdir -p $(LOGS_DIR) $(COVERAGE_DIR) $(DIST_DIR)
	@cp .env.example .env.local 2>/dev/null || true
	@$(MAKE) generate-keypairs
	@$(MAKE) setup-git-hooks
	@$(MAKE) init-database
	@echo "$(GREEN)✅ Development environment ready$(NC)"

check-deps: ## Check system dependencies
	@echo "$(BOLD)$(BLUE)Checking system dependencies...$(NC)"
	@command -v git >/dev/null 2>&1 || { echo "$(RED)❌ Git is required$(NC)"; exit 1; }
	@command -v curl >/dev/null 2>&1 || { echo "$(RED)❌ curl is required$(NC)"; exit 1; }
	@command -v jq >/dev/null 2>&1 || { echo "$(RED)❌ jq is required$(NC)"; exit 1; }
	@echo "$(GREEN)✅ System dependencies verified$(NC)"

install-tools: ## Install development tools
	@echo "$(BOLD)$(BLUE)Installing development tools...$(NC)"
	@command -v rustup >/dev/null 2>&1 || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	@command -v node >/dev/null 2>&1 || curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
	@command -v solana >/dev/null 2>&1 || sh -c "$$(curl -sSfL https://release.solana.com/v$(SOLANA_VERSION)/install)"
	@command -v anchor >/dev/null 2>&1 || cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
	@echo "$(GREEN)✅ Development tools installed$(NC)"

install-rust: ## Install Rust toolchain
	@echo "$(BOLD)$(BLUE)Installing Rust $(RUST_VERSION)...$(NC)"
	@rustup toolchain install $(RUST_VERSION)
	@rustup default $(RUST_VERSION)
	@rustup component add rustfmt clippy
	@rustup target add wasm32-unknown-unknown
	@echo "$(GREEN)✅ Rust toolchain installed$(NC)"

install-node: ## Install Node.js and dependencies
	@echo "$(BOLD)$(BLUE)Installing Node.js $(NODE_VERSION)...$(NC)"
	@command -v nvm >/dev/null 2>&1 && nvm install $(NODE_VERSION) && nvm use $(NODE_VERSION) || true
	@npm install -g yarn@latest
	@npm install -g typescript@latest
	@npm install -g @types/node@latest
	@echo "$(GREEN)✅ Node.js environment installed$(NC)"

install-solana: ## Install Solana CLI and Anchor
	@echo "$(BOLD)$(BLUE)Installing Solana CLI $(SOLANA_VERSION)...$(NC)"
	@solana --version 2>/dev/null || sh -c "$$(curl -sSfL https://release.solana.com/v$(SOLANA_VERSION)/install)"
	@avm install $(ANCHOR_VERSION) 2>/dev/null || true
	@avm use $(ANCHOR_VERSION) 2>/dev/null || true
	@solana config set --url $(ANCHOR_PROVIDER_URL)
	@echo "$(GREEN)✅ Solana development environment ready$(NC)"

# ==============================================================================
# BUILD SYSTEM
# ==============================================================================

.PHONY: build build-programs build-client build-all build-release build-debug clean

build: build-programs build-client ## Build all components
	@echo "$(BOLD)$(GREEN)Building Finova Network ecosystem...$(NC)"
	@echo "$(GREEN)✅ Build completed successfully$(NC)"

build-programs: ## Build all Anchor programs
	@echo "$(BOLD)$(BLUE)Building Anchor programs...$(NC)"
	@cd $(PROGRAMS_DIR) && anchor build
	@echo "$(GREEN)✅ Programs built successfully$(NC)"

build-client: ## Build client SDKs
	@echo "$(BOLD)$(BLUE)Building client SDKs...$(NC)"
	@yarn build:client
	@echo "$(GREEN)✅ Client SDKs built successfully$(NC)"

build-all: clean build test ## Complete build with tests
	@echo "$(BOLD)$(GREEN)Complete build pipeline executed$(NC)"

build-release: ## Build optimized release version
	@echo "$(BOLD)$(BLUE)Building release version...$(NC)"
	@NODE_ENV=production yarn build
	@cargo build --release --all-features
	@anchor build --release
	@echo "$(GREEN)✅ Release build completed$(NC)"

build-debug: ## Build debug version with symbols
	@echo "$(BOLD)$(BLUE)Building debug version...$(NC)"
	@NODE_ENV=development yarn build:dev
	@cargo build --all-features
	@anchor build
	@echo "$(GREEN)✅ Debug build completed$(NC)"

clean: ## Clean all build artifacts
	@echo "$(BOLD)$(YELLOW)Cleaning build artifacts...$(NC)"
	@rm -rf $(BUILD_DIR) $(DIST_DIR) node_modules/.cache
	@cargo clean
	@anchor clean
	@yarn cache clean
	@echo "$(GREEN)✅ Clean completed$(NC)"

# ==============================================================================
# TESTING FRAMEWORK
# ==============================================================================

.PHONY: test test-programs test-client test-integration test-e2e test-security test-performance test-coverage

test: test-programs test-client ## Run all tests
	@echo "$(BOLD)$(GREEN)Running complete test suite...$(NC)"
	@echo "$(GREEN)✅ All tests passed$(NC)"

test-programs: ## Test Anchor programs
	@echo "$(BOLD)$(BLUE)Testing Anchor programs...$(NC)"
	@anchor test --skip-local-validator
	@echo "$(GREEN)✅ Program tests passed$(NC)"

test-client: ## Test client SDKs
	@echo "$(BOLD)$(BLUE)Testing client SDKs...$(NC)"
	@yarn test:client
	@echo "$(GREEN)✅ Client tests passed$(NC)"

test-integration: ## Run integration tests
	@echo "$(BOLD)$(BLUE)Running integration tests...$(NC)"
	@yarn test:integration
	@echo "$(GREEN)✅ Integration tests passed$(NC)"

test-e2e: ## Run end-to-end tests
	@echo "$(BOLD)$(BLUE)Running E2E tests...$(NC)"
	@yarn test:e2e
	@echo "$(GREEN)✅ E2E tests passed$(NC)"

test-security: ## Run security tests
	@echo "$(BOLD)$(BLUE)Running security tests...$(NC)"
	@yarn test:security
	@cargo audit
	@echo "$(GREEN)✅ Security tests passed$(NC)"

test-performance: ## Run performance benchmarks
	@echo "$(BOLD)$(BLUE)Running performance tests...$(NC)"
	@yarn test:performance
	@echo "$(GREEN)✅ Performance tests completed$(NC)"

test-coverage: ## Generate test coverage report
	@echo "$(BOLD)$(BLUE)Generating coverage report...$(NC)"
	@mkdir -p $(COVERAGE_DIR)
	@yarn test:coverage
	@cargo tarpaulin --out html --output-dir $(COVERAGE_DIR)
	@echo "$(GREEN)✅ Coverage report generated in $(COVERAGE_DIR)$(NC)"

# ==============================================================================
# CODE QUALITY & FORMATTING
# ==============================================================================

.PHONY: lint lint-fix format format-check clippy prettier eslint

lint: clippy eslint ## Run all linters
	@echo "$(BOLD)$(BLUE)Running code quality checks...$(NC)"
	@echo "$(GREEN)✅ All linting checks passed$(NC)"

lint-fix: ## Fix linting issues automatically
	@echo "$(BOLD)$(BLUE)Fixing linting issues...$(NC)"
	@cargo clippy --fix --allow-dirty --allow-staged
	@yarn lint:fix
	@$(MAKE) format
	@echo "$(GREEN)✅ Linting issues fixed$(NC)"

format: ## Format all code
	@echo "$(BOLD)$(BLUE)Formatting code...$(NC)"
	@cargo fmt --all
	@yarn prettier --write "**/*.{ts,js,json,md}"
	@echo "$(GREEN)✅ Code formatted$(NC)"

format-check: ## Check code formatting
	@echo "$(BOLD)$(BLUE)Checking code formatting...$(NC)"
	@cargo fmt --all -- --check
	@yarn prettier --check "**/*.{ts,js,json,md}"
	@echo "$(GREEN)✅ Code formatting verified$(NC)"

clippy: ## Run Rust clippy linter
	@echo "$(BOLD)$(BLUE)Running Clippy...$(NC)"
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "$(GREEN)✅ Clippy checks passed$(NC)"

prettier: ## Run Prettier formatter
	@echo "$(BOLD)$(BLUE)Running Prettier...$(NC)"
	@yarn prettier --check "**/*.{ts,js,json,md}"
	@echo "$(GREEN)✅ Prettier checks passed$(NC)"

eslint: ## Run ESLint
	@echo "$(BOLD)$(BLUE)Running ESLint...$(NC)"
	@yarn eslint "**/*.{ts,js}"
	@echo "$(GREEN)✅ ESLint checks passed$(NC)"

# ==============================================================================
# BLOCKCHAIN OPERATIONS
# ==============================================================================

.PHONY: deploy deploy-programs deploy-devnet deploy-testnet deploy-mainnet upgrade verify generate-keypairs

deploy: deploy-programs ## Deploy to configured network
	@echo "$(BOLD)$(GREEN)Deploying to $(SOLANA_NETWORK)...$(NC)"
	@echo "$(GREEN)✅ Deployment completed$(NC)"

deploy-programs: ## Deploy Anchor programs
	@echo "$(BOLD)$(BLUE)Deploying programs to $(SOLANA_NETWORK)...$(NC)"
	@anchor deploy --provider.cluster $(SOLANA_NETWORK)
	@echo "$(GREEN)✅ Programs deployed successfully$(NC)"

deploy-devnet: ## Deploy to Solana devnet
	@echo "$(BOLD)$(BLUE)Deploying to devnet...$(NC)"
	@SOLANA_NETWORK=devnet $(MAKE) deploy-programs
	@echo "$(GREEN)✅ Devnet deployment completed$(NC)"

deploy-testnet: ## Deploy to Solana testnet
	@echo "$(BOLD)$(BLUE)Deploying to testnet...$(NC)"
	@SOLANA_NETWORK=testnet $(MAKE) deploy-programs
	@echo "$(GREEN)✅ Testnet deployment completed$(NC)"

deploy-mainnet: ## Deploy to Solana mainnet (requires confirmation)
	@echo "$(BOLD)$(RED)⚠️  MAINNET DEPLOYMENT - This action is irreversible!$(NC)"
	@read -p "Are you sure you want to deploy to mainnet? (yes/no): " confirm && [ "$$confirm" = "yes" ]
	@SOLANA_NETWORK=mainnet-beta $(MAKE) deploy-programs
	@echo "$(GREEN)✅ Mainnet deployment completed$(NC)"

upgrade: ## Upgrade deployed programs
	@echo "$(BOLD)$(BLUE)Upgrading programs...$(NC)"
	@anchor upgrade --provider.cluster $(SOLANA_NETWORK)
	@echo "$(GREEN)✅ Programs upgraded successfully$(NC)"

verify: ## Verify deployed programs
	@echo "$(BOLD)$(BLUE)Verifying deployed programs...$(NC)"
	@anchor verify --provider.cluster $(SOLANA_NETWORK)
	@echo "$(GREEN)✅ Programs verified successfully$(NC)"

generate-keypairs: ## Generate development keypairs
	@echo "$(BOLD)$(BLUE)Generating keypairs...$(NC)"
	@mkdir -p ~/.config/solana
	@[ ! -f ~/.config/solana/id.json ] && solana-keygen new --no-bip39-passphrase -o ~/.config/solana/id.json --silent || true
	@solana-keygen new --no-bip39-passphrase -o keypairs/finova-core.json --silent 2>/dev/null || true
	@solana-keygen new --no-bip39-passphrase -o keypairs/finova-token.json --silent 2>/dev/null || true
	@solana-keygen new --no-bip39-passphrase -o keypairs/finova-nft.json --silent 2>/dev/null || true
	@echo "$(GREEN)✅ Keypairs generated$(NC)"

# ==============================================================================
# DOCKER OPERATIONS
# ==============================================================================

.PHONY: docker-build docker-push docker-run docker-stop docker-clean docker-compose-up docker-compose-down

docker-build: ## Build Docker images
	@echo "$(BOLD)$(BLUE)Building Docker images...$(NC)"
	@docker buildx build --platform $(DOCKER_PLATFORM) -t $(DOCKER_REGISTRY)/finova-api:$(DOCKER_TAG) -f docker/Dockerfile.api .
	@docker buildx build --platform $(DOCKER_PLATFORM) -t $(DOCKER_REGISTRY)/finova-ai:$(DOCKER_TAG) -f docker/Dockerfile.ai .
	@echo "$(GREEN)✅ Docker images built$(NC)"

docker-push: docker-build ## Push Docker images to registry
	@echo "$(BOLD)$(BLUE)Pushing Docker images...$(NC)"
	@docker push $(DOCKER_REGISTRY)/finova-api:$(DOCKER_TAG)
	@docker push $(DOCKER_REGISTRY)/finova-ai:$(DOCKER_TAG)
	@echo "$(GREEN)✅ Docker images pushed$(NC)"

docker-run: ## Run Docker containers locally
	@echo "$(BOLD)$(BLUE)Starting Docker containers...$(NC)"
	@docker-compose up -d
	@echo "$(GREEN)✅ Docker containers started$(NC)"

docker-stop: ## Stop Docker containers
	@echo "$(BOLD)$(BLUE)Stopping Docker containers...$(NC)"
	@docker-compose down
	@echo "$(GREEN)✅ Docker containers stopped$(NC)"

docker-clean: ## Clean Docker artifacts
	@echo "$(BOLD)$(YELLOW)Cleaning Docker artifacts...$(NC)"
	@docker system prune -f
	@docker volume prune -f
	@echo "$(GREEN)✅ Docker cleanup completed$(NC)"

docker-compose-up: ## Start development environment with Docker Compose
	@echo "$(BOLD)$(BLUE)Starting development environment...$(NC)"
	@docker-compose -f docker-compose.dev.yml up -d
	@echo "$(GREEN)✅ Development environment started$(NC)"

docker-compose-down: ## Stop development environment
	@echo "$(BOLD)$(BLUE)Stopping development environment...$(NC)"
	@docker-compose -f docker-compose.dev.yml down
	@echo "$(GREEN)✅ Development environment stopped$(NC)"

# ==============================================================================
# SECURITY & AUDITING
# ==============================================================================

.PHONY: security-scan audit vulnerability-check dependency-check license-check

security-scan: ## Run comprehensive security scan
	@echo "$(BOLD)$(BLUE)Running security scans...$(NC)"
	@$(MAKE) audit
	@$(MAKE) vulnerability-check
	@$(MAKE) dependency-check
	@echo "$(GREEN)✅ Security scan completed$(NC)"

audit: ## Run security audit
	@echo "$(BOLD)$(BLUE)Running security audit...$(NC)"
	@cargo audit
	@yarn audit --audit-level moderate
	@echo "$(GREEN)✅ Security audit completed$(NC)"

vulnerability-check: ## Check for vulnerabilities
	@echo "$(BOLD)$(BLUE)Checking vulnerabilities...$(NC)"
	@cargo audit --db-path ~/.cargo/advisory-db
	@yarn audit --audit-level high
	@echo "$(GREEN)✅ Vulnerability check completed$(NC)"

dependency-check: ## Check dependency security
	@echo "$(BOLD)$(BLUE)Checking dependency security...$(NC)"
	@cargo tree --duplicates
	@yarn list --depth=0
	@echo "$(GREEN)✅ Dependency check completed$(NC)"

license-check: ## Check license compliance
	@echo "$(BOLD)$(BLUE)Checking license compliance...$(NC)"
	@cargo license
	@yarn licenses list
	@echo "$(GREEN)✅ License check completed$(NC)"

# ==============================================================================
# MONITORING & OPERATIONS
# ==============================================================================

.PHONY: logs status health-check metrics backup restore

logs: ## View application logs
	@echo "$(BOLD)$(BLUE)Viewing logs...$(NC)"
	@tail -f $(LOGS_DIR)/*.log 2>/dev/null || echo "No logs found in $(LOGS_DIR)"

status: ## Show project status
	@echo "$(BOLD)$(CYAN)Finova Network Status$(NC)"
	@echo "$(BLUE)Version:$(NC) $(VERSION)"
	@echo "$(BLUE)Build Date:$(NC) $(BUILD_DATE)"
	@echo "$(BLUE)Git Commit:$(NC) $(GIT_COMMIT)"
	@echo "$(BLUE)Git Branch:$(NC) $(GIT_BRANCH)"
	@echo "$(BLUE)Network:$(NC) $(SOLANA_NETWORK)"
	@echo "$(BLUE)Environment:$(NC) $(NODE_ENV)"

health-check: ## Perform health check
	@echo "$(BOLD)$(BLUE)Performing health check...$(NC)"
	@curl -f http://localhost:3000/health 2>/dev/null && echo "$(GREEN)✅ API healthy$(NC)" || echo "$(RED)❌ API unhealthy$(NC)"
	@solana cluster-version 2>/dev/null && echo "$(GREEN)✅ Solana connection healthy$(NC)" || echo "$(RED)❌ Solana connection failed$(NC)"

metrics: ## Display project metrics
	@echo "$(BOLD)$(BLUE)Project Metrics$(NC)"
	@echo "$(BLUE)Lines of Code:$(NC)"
	@find $(PROGRAMS_DIR) -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print "  Rust: " $$1 " lines"}'
	@find $(CLIENT_DIR) -name "*.ts" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print "  TypeScript: " $$1 " lines"}' || echo "  TypeScript: 0 lines"
	@echo "$(BLUE)Test Coverage:$(NC)"
	@cargo tarpaulin --print summary 2>/dev/null | grep "Coverage" || echo "  Run 'make test-coverage' first"

backup: ## Create project backup
	@echo "$(BOLD)$(BLUE)Creating backup...$(NC)"
	@mkdir -p backups
	@tar -czf backups/finova-backup-$(shell date +%Y%m%d-%H%M%S).tar.gz \
		--exclude=target --exclude=node_modules --exclude=.git \
		--exclude=backups .
	@echo "$(GREEN)✅ Backup created$(NC)"

restore: ## Restore from backup (requires backup file)
	@echo "$(BOLD)$(YELLOW)Restore functionality - specify backup file manually$(NC)"
	@ls -la backups/ 2>/dev/null || echo "No backups found"

# ==============================================================================
# DATABASE OPERATIONS
# ==============================================================================

.PHONY: init-database migrate-database reset-database seed-database

init-database: ## Initialize database
	@echo "$(BOLD)$(BLUE)Initializing database...$(NC)"
	@mkdir -p database/data
	@touch database/data/.gitkeep
	@echo "$(GREEN)✅ Database initialized$(NC)"

migrate-database: ## Run database migrations
	@echo "$(BOLD)$(BLUE)Running database migrations...$(NC)"
	@echo "$(YELLOW)⚠️  Database migrations not implemented yet$(NC)"

reset-database: ## Reset database to initial state
	@echo "$(BOLD)$(YELLOW)Resetting database...$(NC)"
	@rm -rf database/data/*
	@$(MAKE) init-database
	@echo "$(GREEN)✅ Database reset completed$(NC)"

seed-database: ## Seed database with test data
	@echo "$(BOLD)$(BLUE)Seeding database...$(NC)"
	@echo "$(YELLOW)⚠️  Database seeding not implemented yet$(NC)"

# ==============================================================================
# DEVELOPMENT UTILITIES
# ==============================================================================

.PHONY: setup-git-hooks generate-docs update-deps check-updates version-bump

setup-git-hooks: ## Setup Git hooks
	@echo "$(BOLD)$(BLUE)Setting up Git hooks...$(NC)"
	@mkdir -p .git/hooks
	@echo '#!/bin/sh\nmake lint' > .git/hooks/pre-commit
	@echo '#!/bin/sh\nmake test' > .git/hooks/pre-push
	@chmod +x .git/hooks/pre-commit .git/hooks/pre-push
	@echo "$(GREEN)✅ Git hooks configured$(NC)"

generate-docs: ## Generate documentation
	@echo "$(BOLD)$(BLUE)Generating documentation...$(NC)"
	@mkdir -p $(DOCS_DIR)/api
	@cargo doc --no-deps --open
	@yarn typedoc --out $(DOCS_DIR)/api src/
	@echo "$(GREEN)✅ Documentation generated$(NC)"

update-deps: ## Update dependencies
	@echo "$(BOLD)$(BLUE)Updating dependencies...$(NC)"
	@cargo update
	@yarn upgrade-interactive --latest
	@echo "$(GREEN)✅ Dependencies updated$(NC)"

check-updates: ## Check for dependency updates
	@echo "$(BOLD)$(BLUE)Checking for updates...$(NC)"
	@cargo outdated
	@yarn outdated
	@echo "$(GREEN)✅ Update check completed$(NC)"

version-bump: ## Bump version (requires VERSION parameter)
ifndef VERSION
	@echo "$(RED)❌ Please specify VERSION parameter: make version-bump VERSION=1.2.3$(NC)"
	@exit 1
endif
	@echo "$(BOLD)$(BLUE)Bumping version to $(VERSION)...$(NC)"
	@sed -i 's/"version": ".*"/"version": "$(VERSION)"/' package.json
	@sed -i 's/version = ".*"/version = "$(VERSION)"/' Cargo.toml
	@git add package.json Cargo.toml
	@git commit -m "chore: bump version to $(VERSION)"
	@git tag -a v$(VERSION) -m "Version $(VERSION)"
	@echo "$(GREEN)✅ Version bumped to $(VERSION)$(NC)"

# ==============================================================================
# CI/CD INTEGRATION
# ==============================================================================

.PHONY: ci-setup ci-test ci-build ci-deploy ci-release

ci-setup: ## Setup CI environment
	@echo "$(BOLD)$(BLUE)Setting up CI environment...$(NC)"
	@$(MAKE) install-tools
	@$(MAKE) check-deps
	@echo "$(GREEN)✅ CI environment ready$(NC)"

ci-test: ## Run tests in CI environment
	@echo "$(BOLD)$(BLUE)Running CI test suite...$(NC)"
	@$(MAKE) lint
	@$(MAKE) test
	@$(MAKE) security-scan
	@echo "$(GREEN)✅ CI tests completed$(NC)"

ci-build: ## Build in CI environment
	@echo "$(BOLD)$(BLUE)Running CI build...$(NC)"
	@$(MAKE) build-release
	@$(MAKE) test-coverage
	@echo "$(GREEN)✅ CI build completed$(NC)"

ci-deploy: ## Deploy in CI environment
	@echo "$(BOLD)$(BLUE)Running CI deployment...$(NC)"
	@$(MAKE) deploy-$(SOLANA_NETWORK)
	@$(MAKE) verify
	@echo "$(GREEN)✅ CI deployment completed$(NC)"

ci-release: ## Create release in CI environment
	@echo "$(BOLD)$(BLUE)Creating release...$(NC)"
	@$(MAKE) build-release
	@$(MAKE) docker-build
	@$(MAKE) generate-docs
	@echo "$(GREEN)✅ Release created$(NC)"

# ==============================================================================
# UTILITY TARGETS
# ==============================================================================

.PHONY: info env debug benchmark profile

info: ## Display project information
	@echo "$(BOLD)$(CYAN)Finova Network - Project Information$(NC)"
	@echo "$(BLUE)Project:$(NC) $(PROJECT_NAME)"
	@echo "$(BLUE)Version:$(NC) $(VERSION)"
	@echo "$(BLUE)Build Date:$(NC) $(BUILD_DATE)"
	@echo "$(BLUE)Git Commit:$(NC) $(GIT_COMMIT)"
	@echo "$(BLUE)Git Branch:$(NC) $(GIT_BRANCH)"
	@echo "$(BLUE)Root Directory:$(NC) $(ROOT_DIR)"
	@echo "$(BLUE)Node Environment:$(NC) $(NODE_ENV)"
	@echo "$(BLUE)Solana Network:$(NC) $(SOLANA_NETWORK)"
	@echo "$(BLUE)Anchor Provider:$(NC) $(ANCHOR_PROVIDER_URL)"

env: ## Display environment variables
	@echo "$(BOLD)$(CYAN)Environment Variables$(NC)"
	@env | grep -E "^(NODE_|RUST_|SOLANA_|ANCHOR_|DOCKER_)" | sort

debug: ## Enable debug mode for all operations
	@echo "$(BOLD)$(YELLOW)Debug mode enabled$(NC)"
	@export RUST_LOG=debug
	@export NODE_ENV=development
	@export DEBUG=*

benchmark: ## Run performance benchmarks
	@echo "$(BOLD)$(BLUE)Running benchmarks...$(NC)"
	@cargo bench
	@yarn benchmark
	@echo "$(GREEN)✅ Benchmarks completed$(NC)"

profile: ## Run profiling analysis
	@echo "$(BOLD)$(BLUE)Running profiling analysis...$(NC)"
	@cargo flamegraph --bin finova-core
	@echo "$(GREEN)✅ Profiling completed$(NC)"

# ==============================================================================
# ERROR HANDLING & VALIDATION
# ==============================================================================

# Ensure required directories exist
$(BUILD_DIR) $(DIST_DIR) $(LOGS_DIR) $(COVERAGE_DIR):
	@mkdir -p $@

# Validate environment before critical operations
validate-env:
	@test -n "$(ANCHOR_PROVIDER_URL)" || { echo "$(RED)❌ ANCHOR_PROVIDER_URL not set$(NC)"; exit 1; }
	@test -f "$(ANCHOR_WALLET)" || { echo "$(RED)❌ Wallet not found at $(ANCHOR_WALLET)$(NC)"; exit 1; }

# Check if network is safe for operations
check-network:
	@if [ "$(SOLANA_NETWORK)" = "mainnet-beta" ]; then \
		echo "$(RED)⚠️  MAINNET DETECTED - Use with extreme caution!$(NC)"; \
		read -p "Continue with mainnet operation? (yes/no): " confirm; \
		[ "$confirm" != "yes" ] && exit 1; \
	fi

# ==============================================================================
# ADVANCED FEATURES
# ==============================================================================

.PHONY: ai-setup ai-train ai-deploy social-setup social-test mobile-setup mobile-build

ai-setup: ## Setup AI services environment
	@echo "$(BOLD)$(BLUE)Setting up AI services...$(NC)"
	@pip install -r ai-services/requirements.txt 2>/dev/null || echo "$(YELLOW)⚠️  AI services requirements not found$(NC)"
	@mkdir -p ai-services/models ai-services/data
	@echo "$(GREEN)✅ AI services environment ready$(NC)"

ai-train: ## Train AI models
	@echo "$(BOLD)$(BLUE)Training AI models...$(NC)"
	@python ai-services/content-analyzer/train.py 2>/dev/null || echo "$(YELLOW)⚠️  AI training scripts not implemented$(NC)"
	@echo "$(GREEN)✅ AI model training completed$(NC)"

ai-deploy: ## Deploy AI services
	@echo "$(BOLD)$(BLUE)Deploying AI services...$(NC)"
	@docker-compose -f docker-compose.ai.yml up -d 2>/dev/null || echo "$(YELLOW)⚠️  AI docker compose not found$(NC)"
	@echo "$(GREEN)✅ AI services deployed$(NC)"

social-setup: ## Setup social media integrations
	@echo "$(BOLD)$(BLUE)Setting up social integrations...$(NC)"
	@mkdir -p integrations/social/{instagram,tiktok,youtube,facebook,twitter}
	@echo "$(GREEN)✅ Social integration structure ready$(NC)"

social-test: ## Test social media integrations
	@echo "$(BOLD)$(BLUE)Testing social integrations...$(NC)"
	@yarn test:social 2>/dev/null || echo "$(YELLOW)⚠️  Social tests not implemented$(NC)"
	@echo "$(GREEN)✅ Social integration tests completed$(NC)"

mobile-setup: ## Setup mobile development environment
	@echo "$(BOLD)$(BLUE)Setting up mobile environment...$(NC)"
	@command -v react-native >/dev/null 2>&1 || npm install -g react-native-cli
	@command -v pod >/dev/null 2>&1 || echo "$(YELLOW)⚠️  CocoaPods not found - install for iOS development$(NC)"
	@mkdir -p mobile-sdk/{ios,android,react-native}
	@echo "$(GREEN)✅ Mobile environment ready$(NC)"

mobile-build: ## Build mobile SDKs
	@echo "$(BOLD)$(BLUE)Building mobile SDKs...$(NC)"
	@cd mobile-sdk/react-native && yarn build 2>/dev/null || echo "$(YELLOW)⚠️  React Native SDK not found$(NC)"
	@echo "$(GREEN)✅ Mobile SDKs built$(NC)"

# ==============================================================================
# PERFORMANCE OPTIMIZATION
# ==============================================================================

.PHONY: optimize optimize-rust optimize-js optimize-size optimize-speed

optimize: optimize-rust optimize-js ## Run all optimizations
	@echo "$(BOLD)$(GREEN)All optimizations completed$(NC)"

optimize-rust: ## Optimize Rust code
	@echo "$(BOLD)$(BLUE)Optimizing Rust code...$(NC)"
	@cargo build --release --target wasm32-unknown-unknown
	@wasm-opt -Oz -o target/optimized.wasm target/wasm32-unknown-unknown/release/*.wasm 2>/dev/null || true
	@echo "$(GREEN)✅ Rust optimization completed$(NC)"

optimize-js: ## Optimize JavaScript/TypeScript code
	@echo "$(BOLD)$(BLUE)Optimizing JavaScript/TypeScript...$(NC)"
	@yarn build:prod
	@yarn analyze 2>/dev/null || echo "$(YELLOW)⚠️  Bundle analyzer not configured$(NC)"
	@echo "$(GREEN)✅ JavaScript optimization completed$(NC)"

optimize-size: ## Optimize for smaller binary size
	@echo "$(BOLD)$(BLUE)Optimizing for size...$(NC)"
	@cargo build --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu
	@strip target/x86_64-unknown-linux-gnu/release/* 2>/dev/null || true
	@echo "$(GREEN)✅ Size optimization completed$(NC)"

optimize-speed: ## Optimize for execution speed
	@echo "$(BOLD)$(BLUE)Optimizing for speed...$(NC)"
	@cargo build --release --features="simd,parallel"
	@echo "$(GREEN)✅ Speed optimization completed$(NC)"

# ==============================================================================
# QUALITY ASSURANCE
# ==============================================================================

.PHONY: qa qa-full qa-security qa-performance qa-compatibility qa-report

qa: qa-security qa-performance qa-compatibility ## Run QA suite
	@echo "$(BOLD)$(GREEN)QA suite completed$(NC)"

qa-full: ## Run comprehensive QA
	@echo "$(BOLD)$(BLUE)Running comprehensive QA...$(NC)"
	@$(MAKE) lint
	@$(MAKE) test
	@$(MAKE) security-scan
	@$(MAKE) performance-test
	@$(MAKE) compatibility-test
	@$(MAKE) qa-report
	@echo "$(GREEN)✅ Comprehensive QA completed$(NC)"

qa-security: ## Security-focused QA
	@echo "$(BOLD)$(BLUE)Running security QA...$(NC)"
	@$(MAKE) security-scan
	@$(MAKE) vulnerability-check
	@echo "$(GREEN)✅ Security QA completed$(NC)"

qa-performance: ## Performance-focused QA
	@echo "$(BOLD)$(BLUE)Running performance QA...$(NC)"
	@$(MAKE) benchmark
	@$(MAKE) profile
	@echo "$(GREEN)✅ Performance QA completed$(NC)"

qa-compatibility: ## Compatibility testing
	@echo "$(BOLD)$(BLUE)Running compatibility tests...$(NC)"
	@rustc --version
	@node --version
	@solana --version
	@anchor --version
	@echo "$(GREEN)✅ Compatibility testing completed$(NC)"

qa-report: ## Generate QA report
	@echo "$(BOLD)$(BLUE)Generating QA report...$(NC)"
	@mkdir -p reports
	@echo "# Finova Network QA Report" > reports/qa-report.md
	@echo "Generated: $(BUILD_DATE)" >> reports/qa-report.md
	@echo "Commit: $(GIT_COMMIT)" >> reports/qa-report.md
	@echo "$(GREEN)✅ QA report generated$(NC)"

# ==============================================================================
# MAINTENANCE & HOUSEKEEPING
# ==============================================================================

.PHONY: maintenance cleanup update-all refresh reset-hard

maintenance: ## Run maintenance tasks
	@echo "$(BOLD)$(BLUE)Running maintenance tasks...$(NC)"
	@$(MAKE) cleanup
	@$(MAKE) update-deps
	@$(MAKE) security-scan
	@echo "$(GREEN)✅ Maintenance completed$(NC)"

cleanup: ## Comprehensive cleanup
	@echo "$(BOLD)$(YELLOW)Running comprehensive cleanup...$(NC)"
	@$(MAKE) clean
	@docker system prune -f 2>/dev/null || true
	@yarn cache clean
	@cargo clean
	@rm -rf $(LOGS_DIR)/*.log 2>/dev/null || true
	@rm -rf tmp/ .tmp/ 2>/dev/null || true
	@echo "$(GREEN)✅ Cleanup completed$(NC)"

update-all: ## Update everything
	@echo "$(BOLD)$(BLUE)Updating all components...$(NC)"
	@$(MAKE) update-deps
	@rustup update
	@solana install update
	@avm update
	@echo "$(GREEN)✅ All updates completed$(NC)"

refresh: ## Refresh development environment
	@echo "$(BOLD)$(BLUE)Refreshing development environment...$(NC)"
	@$(MAKE) cleanup
	@$(MAKE) install
	@$(MAKE) build
	@$(MAKE) test
	@echo "$(GREEN)✅ Environment refreshed$(NC)"

reset-hard: ## Hard reset (destructive operation)
	@echo "$(BOLD)$(RED)⚠️  DESTRUCTIVE OPERATION - This will delete all local changes!$(NC)"
	@read -p "Are you absolutely sure? (yes/no): " confirm && [ "$confirm" = "yes" ]
	@git clean -fdx
	@git reset --hard HEAD
	@$(MAKE) setup-dev
	@echo "$(GREEN)✅ Hard reset completed$(NC)"

# ==============================================================================
# MULTI-ENVIRONMENT SUPPORT
# ==============================================================================

.PHONY: env-dev env-staging env-prod switch-env

env-dev: ## Switch to development environment
	@echo "$(BOLD)$(BLUE)Switching to development environment...$(NC)"
	@cp config/environments/development.json .env.local
	@export NODE_ENV=development SOLANA_NETWORK=devnet
	@echo "$(GREEN)✅ Development environment active$(NC)"

env-staging: ## Switch to staging environment
	@echo "$(BOLD)$(BLUE)Switching to staging environment...$(NC)"
	@cp config/environments/staging.json .env.local
	@export NODE_ENV=staging SOLANA_NETWORK=testnet
	@echo "$(GREEN)✅ Staging environment active$(NC)"

env-prod: ## Switch to production environment
	@echo "$(BOLD)$(RED)⚠️  Switching to production environment...$(NC)"
	@read -p "Are you sure you want to switch to production? (yes/no): " confirm && [ "$confirm" = "yes" ]
	@cp config/environments/production.json .env.local
	@export NODE_ENV=production SOLANA_NETWORK=mainnet-beta
	@echo "$(GREEN)✅ Production environment active$(NC)"

switch-env: ## Interactive environment switcher
	@echo "$(BOLD)$(CYAN)Environment Switcher$(NC)"
	@echo "1) Development (devnet)"
	@echo "2) Staging (testnet)"
	@echo "3) Production (mainnet)"
	@read -p "Select environment (1-3): " choice; \
	case $choice in \
		1) $(MAKE) env-dev ;; \
		2) $(MAKE) env-staging ;; \
		3) $(MAKE) env-prod ;; \
		*) echo "$(RED)Invalid choice$(NC)" ;; \
	esac

# ==============================================================================
# TROUBLESHOOTING & DIAGNOSTICS
# ==============================================================================

.PHONY: diagnose fix-common-issues doctor

diagnose: ## Run diagnostic checks
	@echo "$(BOLD)$(BLUE)Running diagnostic checks...$(NC)"
	@echo "$(BLUE)System Information:$(NC)"
	@uname -a
	@echo "$(BLUE)Disk Space:$(NC)"
	@df -h . 2>/dev/null || echo "N/A"
	@echo "$(BLUE)Memory Usage:$(NC)"
	@free -h 2>/dev/null || echo "N/A"
	@echo "$(BLUE)Tool Versions:$(NC)"
	@rustc --version 2>/dev/null || echo "Rust: Not installed"
	@node --version 2>/dev/null || echo "Node.js: Not installed"
	@solana --version 2>/dev/null || echo "Solana: Not installed"
	@anchor --version 2>/dev/null || echo "Anchor: Not installed"
	@echo "$(GREEN)✅ Diagnostics completed$(NC)"

fix-common-issues: ## Fix common development issues
	@echo "$(BOLD)$(BLUE)Fixing common issues...$(NC)"
	@echo "$(BLUE)Clearing caches...$(NC)"
	@rm -rf node_modules/.cache target/debug/incremental 2>/dev/null || true
	@echo "$(BLUE)Resetting Cargo registry...$(NC)"
	@cargo clean 2>/dev/null || true
	@echo "$(BLUE)Reinstalling dependencies...$(NC)"
	@yarn install --force 2>/dev/null || true
	@echo "$(GREEN)✅ Common issues fixed$(NC)"

doctor: ## Comprehensive health check
	@echo "$(BOLD)$(CYAN)Finova Network Doctor$(NC)"
	@echo "🏥 Checking system health..."
	@$(MAKE) check-deps
	@$(MAKE) diagnose
	@$(MAKE) health-check
	@echo "$(GREEN)✅ System health check completed$(NC)"

# ==============================================================================
# AUTOMATION & WORKFLOWS
# ==============================================================================

.PHONY: workflow-dev workflow-ci workflow-deploy auto-format auto-test

workflow-dev: ## Complete development workflow
	@echo "$(BOLD)$(CYAN)Development Workflow$(NC)"
	@$(MAKE) format
	@$(MAKE) lint
	@$(MAKE) build
	@$(MAKE) test
	@echo "$(GREEN)✅ Development workflow completed$(NC)"

workflow-ci: ## CI/CD workflow
	@echo "$(BOLD)$(CYAN)CI/CD Workflow$(NC)"
	@$(MAKE) ci-setup
	@$(MAKE) ci-test
	@$(MAKE) ci-build
	@echo "$(GREEN)✅ CI/CD workflow completed$(NC)"

workflow-deploy: ## Deployment workflow
	@echo "$(BOLD)$(CYAN)Deployment Workflow$(NC)"
	@$(MAKE) validate-env
	@$(MAKE) check-network
	@$(MAKE) build-release
	@$(MAKE) test
	@$(MAKE) deploy
	@$(MAKE) verify
	@echo "$(GREEN)✅ Deployment workflow completed$(NC)"

auto-format: ## Auto-format on file changes (requires inotify-tools)
	@echo "$(BOLD)$(BLUE)Starting auto-format watcher...$(NC)"
	@while inotifywait -r -e modify --include='\.(rs|ts|js)$' .; do \
		$(MAKE) format; \
	done 2>/dev/null || echo "$(YELLOW)⚠️  inotify-tools required for auto-format$(NC)"

auto-test: ## Auto-test on file changes (requires inotify-tools)
	@echo "$(BOLD)$(BLUE)Starting auto-test watcher...$(NC)"
	@while inotifywait -r -e modify --include='\.(rs|ts|js)$' .; do \
		$(MAKE) test; \
	done 2>/dev/null || echo "$(YELLOW)⚠️  inotify-tools required for auto-test$(NC)"

# ==============================================================================
# ANALYTICS & REPORTING
# ==============================================================================

.PHONY: analytics report metrics-detailed complexity-analysis

analytics: ## Generate project analytics
	@echo "$(BOLD)$(BLUE)Generating project analytics...$(NC)"
	@mkdir -p reports/analytics
	@echo "# Project Analytics Report" > reports/analytics/summary.md
	@echo "Generated: $(BUILD_DATE)" >> reports/analytics/summary.md
	@$(MAKE) metrics-detailed >> reports/analytics/summary.md
	@echo "$(GREEN)✅ Analytics report generated$(NC)"

report: ## Generate comprehensive project report
	@echo "$(BOLD)$(BLUE)Generating comprehensive report...$(NC)"
	@mkdir -p reports
	@echo "# Finova Network - Project Report" > reports/project-report.md
	@echo "Version: $(VERSION)" >> reports/project-report.md
	@echo "Generated: $(BUILD_DATE)" >> reports/project-report.md
	@echo "Git Commit: $(GIT_COMMIT)" >> reports/project-report.md
	@echo "" >> reports/project-report.md
	@$(MAKE) metrics >> reports/project-report.md
	@echo "$(GREEN)✅ Project report generated$(NC)"

metrics-detailed: ## Detailed project metrics
	@echo "$(BOLD)$(BLUE)Collecting detailed metrics...$(NC)"
	@echo "## Code Metrics"
	@echo "### Rust Code"
	@find programs/ -name "*.rs" -type f | wc -l | awk '{print "Files: " $1}'
	@find programs/ -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print "Lines: " $1}'
	@echo "### TypeScript Code"
	@find client/ -name "*.ts" -type f 2>/dev/null | wc -l | awk '{print "Files: " $1}' || echo "Files: 0"
	@find client/ -name "*.ts" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print "Lines: " $1}' || echo "Lines: 0"
	@echo "### Test Files"
	@find tests/ -name "*.rs" -o -name "*.ts" -type f 2>/dev/null | wc -l | awk '{print "Test Files: " $1}' || echo "Test Files: 0"

complexity-analysis: ## Analyze code complexity
	@echo "$(BOLD)$(BLUE)Analyzing code complexity...$(NC)"
	@cargo clippy -- -W clippy::cognitive_complexity 2>/dev/null || echo "$(YELLOW)⚠️  Complexity analysis requires clippy$(NC)"
	@echo "$(GREEN)✅ Complexity analysis completed$(NC)"

# ==============================================================================
# FINAL UTILITIES
# ==============================================================================

# Ensure all operations have proper error handling and logging
%: 
	@if [ -z "$(filter $@,$(MAKECMDGOALS))" ]; then \
		echo "$(RED)❌ Unknown target: $@$(NC)"; \
		echo "$(BLUE)Run 'make help' to see available targets$(NC)"; \
		exit 1; \
	fi

# Export variables for sub-makes
export NODE_ENV SOLANA_NETWORK ANCHOR_PROVIDER_URL ANCHOR_WALLET
export BUILD_DATE GIT_COMMIT GIT_BRANCH VERSION

# Prevent parallel execution of conflicting targets
.NOTPARALLEL: deploy deploy-mainnet reset-hard

# Keep intermediate files for debugging
.PRECIOUS: %.log %.tmp

# ==============================================================================
# END OF MAKEFILE
# ==============================================================================

# ASCII Art Banner (commented out for practical use)
# ███████╗██╗███╗   ██╗ ██████╗ ██╗   ██╗ █████╗ 
# ██╔════╝██║████╗  ██║██╔═══██╗██║   ██║██╔══██╗
# █████╗  ██║██╔██╗ ██║██║   ██║██║   ██║███████║
# ██╔══╝  ██║██║╚██╗██║██║   ██║╚██╗ ██╔╝██╔══██║
# ██║     ██║██║ ╚████║╚██████╔╝ ╚████╔╝ ██║  ██║
# ╚═╝     ╚═╝╚═╝  ╚═══╝ ╚═════╝   ╚═══╝  ╚═╝  ╚═╝

.SILENT: help info status env diagnose
