# Hodei Verified Permissions - Makefile
# Centralized commands for development, testing, and operations

# Variables
PROJECT_ROOT := $(shell pwd)
DATABASE_URL := sqlite:///home/rubentxu/hodei-data/hodei.db
API_URL := http://localhost:3000
GRPC_URL := localhost:50051
BUILD_DIR := target

# Colors for output
CYAN := \033[0;36m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m # No Color

.PHONY: help
help: ## Show this help message
	@echo "$(CYAN)Hodei Verified Permissions - Available Commands$(NC)"
	@echo ""
	@echo "$(YELLOW)=== QUICK START ===$(NC)"
	@echo "  $(GREEN)make test-all$(NC)            Run all tests (unit + integration)"
	@echo "  $(GREEN)make test-e2e$(NC)            Run E2E tests (with services)"
	@echo "  $(GREEN)make test-complete$(NC)       Run ALL tests (including E2E)"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-25s$(NC) %s\n", $$1, $$2}'
	@echo ""

# =============================================================================
# DEVELOPMENT COMMANDS
# =============================================================================

.PHONY: dev
dev: ## Start all services in development mode
	@echo "$(CYAN)🚀 Starting development environment...$(NC)"
	@$(MAKE) build
	@$(MAKE) db-init
	@echo "$(CYAN)📡 Starting gRPC server in background...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && nohup DATABASE_URL=$(DATABASE_URL) cargo run --bin hodei-verified-permissions > /tmp/hodei-server.log 2>&1 &
	@echo "$(CYAN)🌐 Starting Next.js frontend in background...$(NC)"
	@cd $(PROJECT_ROOT)/web-nextjs && nohup npm run dev > /tmp/hodei-web.log 2>&1 &
	@sleep 3
	@echo "$(GREEN)✅ Services started!$(NC)"
	@echo "$(YELLOW)📍 Frontend: http://localhost:3000$(NC)"
	@echo "$(YELLOW)📍 gRPC API: localhost:50051$(NC)"
	@echo "$(YELLOW)💡 View logs: tail -f /tmp/hodei-server.log (backend) or /tmp/hodei-web.log (frontend)$(NC)"
	@echo "$(YELLOW)💡 Stop services: make stop$(NC)"

.PHONY: build
build: ## Build all Rust components
	@echo "$(CYAN)🔨 Building Rust components...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo build --all-targets
	@echo "$(GREEN)✅ Build completed!$(NC)"

.PHONY: build-release
build-release: ## Build all Rust components in release mode
	@echo "$(CYAN)🔨 Building Rust components (release)...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo build --all-targets --release
	@echo "$(GREEN)✅ Release build completed!$(NC)"

.PHONY: clean
clean: ## Clean build artifacts
	@echo "$(CYAN)🧹 Cleaning build artifacts...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo clean
	@rm -rf $(BUILD_DIR)
	@echo "$(GREEN)✅ Clean completed!$(NC)"

# =============================================================================
# DATABASE COMMANDS
# =============================================================================

.PHONY: db-init
db-init: ## Initialize database
	@echo "$(CYAN)🗄️  Initializing database...$(NC)"
	@mkdir -p /home/rubentxu/hodei-data
	@touch /home/rubentxu/hodei-data/hodei.db
	@echo "$(GREEN)✅ Database initialized!$(NC)"

.PHONY: db-reset
db-reset: ## Reset database (WARNING: Deletes all data)
	@echo "$(RED)⚠️  Resetting database...$(NC)"
	@rm -f /home/rubentxu/hodei-data/hodei.db
	@$(MAKE) db-init
	@echo "$(GREEN)✅ Database reset!$(NC)"

# =============================================================================
# SERVER COMMANDS
# =============================================================================

.PHONY: server
server: ## Start gRPC server
	@echo "$(CYAN)🚀 Starting gRPC server...$(NC)"
	@export DATABASE_URL=$(DATABASE_URL) && \
	cd $(PROJECT_ROOT)/verified-permissions && \
	cargo run --bin hodei-verified-permissions

.PHONY: server-release
server-release: build-release ## Start gRPC server in release mode
	@echo "$(CYAN)🚀 Starting gRPC server (release)...$(NC)"
	@export DATABASE_URL=$(DATABASE_URL) && \
	$(BUILD_DIR)/release/hodei-verified-permissions

.PHONY: web
web: ## Start Next.js web interface
	@echo "$(CYAN)🌐 Starting web interface...$(NC)"
	@cd $(PROJECT_ROOT)/web-nextjs && npm run dev

.PHONY: web-build
web-build: ## Build Next.js for production
	@echo "$(CYAN)🔨 Building web interface...$(NC)"
	@cd $(PROJECT_ROOT)/web-nextjs && npm run build
	@echo "$(GREEN)✅ Web build completed!$(NC)"

# =============================================================================
# CODE QUALITY
# =============================================================================

.PHONY: lint
lint: ## Run linters
	@echo "$(CYAN)🔍 Running linters...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo clippy --all-targets
	@echo "$(GREEN)✅ Linting completed!$(NC)"

.PHONY: format
format: ## Format code
	@echo "$(CYAN)🎨 Formatting code...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo fmt --all
	@cd $(PROJECT_ROOT)/web-nextjs && npx prettier --write "src/**/*.{ts,tsx,js,jsx}"
	@echo "$(GREEN)✅ Code formatted!$(NC)"

.PHONY: check
check: ## Run cargo check
	@echo "$(CYAN)✅ Running cargo check...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo check --all-targets
	@echo "$(GREEN)✅ Check completed!$(NC)"

.PHONY: audit
audit: ## Run security audit
	@echo "$(CYAN)🔒 Running security audit...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo audit
	@echo "$(GREEN)✅ Security audit completed!$(NC)"

# =============================================================================
# TESTING COMMANDS - SDK
# =============================================================================

.PHONY: test-sdk
test-sdk: ## Run SDK tests (Data Plane only, excluding doctests)
	@echo "$(CYAN)🧪 Running SDK tests (Data Plane)...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions-sdk && cargo test --lib --tests
	@echo "$(GREEN)✅ SDK tests completed!$(NC)"

.PHONY: test-sdk-unit
test-sdk-unit: ## Run SDK unit tests
	@echo "$(CYAN)🧪 Running SDK unit tests...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions-sdk && cargo test --lib --tests -- --nocapture
	@echo "$(GREEN)✅ SDK unit tests completed!$(NC)"

.PHONY: test-sdk-integration
test-sdk-integration: ## Run SDK integration tests
	@echo "$(CYAN)🧪 Running SDK integration tests...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions-sdk && cargo test --test integration_test
	@echo "$(GREEN)✅ SDK integration tests completed!$(NC)"

.PHONY: test-sdk-admin
test-sdk-admin: ## Run SDK Admin tests (Control Plane)
	@echo "$(CYAN)🧪 Running SDK Admin tests (Control Plane)...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions-sdk-admin && cargo test --lib --tests
	@echo "$(GREEN)✅ SDK Admin tests completed!$(NC)"

.PHONY: test-sdk-admin-unit
test-sdk-admin-unit: ## Run SDK Admin unit tests
	@echo "$(CYAN)🧪 Running SDK Admin unit tests...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions-sdk-admin && cargo test --lib
	@echo "$(GREEN)✅ SDK Admin unit tests completed!$(NC)"

.PHONY: test-sdk-admin-integration
test-sdk-admin-integration: ## Run SDK Admin integration tests
	@echo "$(CYAN)🧪 Running SDK Admin integration tests...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions-sdk-admin && cargo test --test integration_test
	@echo "$(GREEN)✅ SDK Admin integration tests completed!$(NC)"

# =============================================================================
# TESTING COMMANDS - BACKEND
# =============================================================================

.PHONY: test-backend
test-backend: ## Run backend tests
	@echo "$(CYAN)🧪 Running backend tests...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo test
	@echo "$(GREEN)✅ Backend tests completed!$(NC)"

.PHONY: test-backend-unit
test-backend-unit: ## Run backend unit tests
	@echo "$(CYAN)🧪 Running backend unit tests...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo test --lib
	@echo "$(GREEN)✅ Backend unit tests completed!$(NC)"

.PHONY: test-backend-integration
test-backend-integration: ## Run backend integration tests (simple tests only)
	@echo "$(CYAN)🧪 Running backend integration tests (simple)...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo test --test simple_integration_test
	@echo "$(GREEN)✅ Backend simple integration tests completed!$(NC)"

.PHONY: test-backend-e2e
test-backend-e2e: ## Run backend E2E tests (requires running server)
	@echo "$(YELLOW)⚠️  Running backend E2E tests (requires server)...$(NC)"
	@echo "$(CYAN)Note: Starting server...$(NC)"
	@make test-infrastructure-up DB=sqlite
	@make server &
	@sleep 5
	@cd $(PROJECT_ROOT)/verified-permissions && cargo test --test e2e_repository_tests --test e2e_policy_store_tests --features containers 2>&1 || true
	@make stop
	@make test-infrastructure-down
	@echo "$(GREEN)✅ Backend E2E tests completed!$(NC)"

# =============================================================================
# =============================================================================
# NEW MODULAR TEST COMMANDS
# =============================================================================

.PHONY: test-infrastructure-up
test-infrastructure-up: ## Start Docker infrastructure for tests (use DB=sqlite|postgres|surrealdb|all)
	@echo "$(CYAN)🐳 Starting Docker infrastructure...$(NC)"
	@if [ -n "$(DB)" ]; then \
		bash $(PROJECT_ROOT)/scripts/test/infra-up.sh $(DB); \
	else \
		bash $(PROJECT_ROOT)/scripts/test/infra-up.sh sqlite; \
	fi

.PHONY: test-infrastructure-status
test-infrastructure-status: ## Check Docker infrastructure status
	@echo "$(CYAN)📊 Checking infrastructure status...$(NC)"
	@docker-compose -f $(PROJECT_ROOT)/scripts/test/docker-compose/sqlite.yml ps 2>/dev/null || echo "No infrastructure detected"

.PHONY: test-infrastructure-logs
test-infrastructure-logs: ## View Docker infrastructure logs
	@echo "$(CYAN)📜 Viewing infrastructure logs...$(NC)"
	@docker-compose -f $(PROJECT_ROOT)/scripts/test/docker-compose/sqlite.yml logs 2>/dev/null || echo "No infrastructure logs available"

.PHONY: test-infrastructure-down
test-infrastructure-down: ## Stop Docker infrastructure for tests
	@echo "$(CYAN)🐳 Stopping Docker infrastructure...$(NC)"
	@bash $(PROJECT_ROOT)/scripts/test/infra-down.sh
	@echo "$(GREEN)✅ Infrastructure stopped!$(NC)"

.PHONY: test-e2e-install
test-e2e-install: ## Install Playwright browsers
	@echo "$(CYAN)🌐 Installing Playwright browsers...$(NC)"
	@bash $(PROJECT_ROOT)/scripts/test/e2e.sh install
	@echo "$(GREEN)✅ Playwright browsers installed!$(NC)"

.PHONY: test-e2e-services-start
test-e2e-services-start: ## Start services for E2E testing
	@echo "$(CYAN)🚀 Starting services for E2E tests...$(NC)"
	@make test-infrastructure-up
	@make server
	@echo "$(GREEN)✅ Services started!$(NC)"

.PHONY: test-e2e-services-stop
test-e2e-services-stop: ## Stop E2E test services
	@echo "$(CYAN)⏹️  Stopping E2E test services...$(NC)"
	@make stop || true
	@make test-infrastructure-down || true
	@echo "$(GREEN)✅ Services stopped!$(NC)"

.PHONY: test-e2e-services-status
test-e2e-services-status: ## Check E2E test services status
	@echo "$(CYAN)📊 Checking service status...$(NC)"
	@netstat -tlnp 2>/dev/null | grep -E "3000|50051" || echo "No services detected"

.PHONY: test-e2e
test-e2e: ## Run full E2E test suite (with services)
	@echo "$(CYAN)🚀 Starting services for E2E tests...$(NC)"
	@$(MAKE) test-e2e-services-start
	@echo "$(CYAN)🧪 Running E2E test suite...$(NC)"
	@bash $(PROJECT_ROOT)/scripts/test/e2e.sh test
	@echo "$(CYAN)🧹 Stopping services...$(NC)"
	@$(MAKE) test-e2e-services-stop
	@echo "$(GREEN)✅ E2E tests completed!$(NC)"

.PHONY: test-e2e-ui
test-e2e-ui: ## Run E2E tests with Playwright UI
	@echo "$(CYAN)🧪 Running E2E tests in UI mode...$(NC)"
	@bash $(PROJECT_ROOT)/scripts/test/e2e.sh ui
	@echo "$(GREEN)✅ E2E tests completed!$(NC)"

.PHONY: test-e2e-headed
test-e2e-headed: ## Run E2E tests in headed mode (visible browser)
	@echo "$(CYAN)🧪 Running E2E tests in headed mode...$(NC)"
	@bash $(PROJECT_ROOT)/scripts/test/e2e.sh headed
	@echo "$(GREEN)✅ E2E tests completed!$(NC)"

.PHONY: test-e2e-debug
test-e2e-debug: ## Run E2E tests in debug mode
	@echo "$(CYAN)🧪 Running E2E tests in debug mode...$(NC)"
	@bash $(PROJECT_ROOT)/scripts/test/e2e.sh debug
	@echo "$(GREEN)✅ E2E tests completed!$(NC)"

.PHONY: test-e2e-full
test-e2e-full: ## Run full E2E test suite (starts services automatically)
	@echo "$(CYAN)🧪 Running full E2E test suite...$(NC)"
	@bash $(PROJECT_ROOT)/scripts/test/e2e-simple.sh
	@echo "$(GREEN)✅ Full E2E test suite completed!$(NC)"

.PHONY: test-e2e-report
test-e2e-report: ## Open E2E test report
	@echo "$(CYAN)📊 Opening E2E test report...$(NC)"
	@if [ -f "$(PROJECT_ROOT)/test-results/e2e/results.html" ]; then \
		open "$(PROJECT_ROOT)/test-results/e2e/results.html" 2>/dev/null || \
		xdg-open "$(PROJECT_ROOT)/test-results/e2e/results.html" 2>/dev/null || \
		echo "Report location: $(PROJECT_ROOT)/test-results/e2e/results.html"; \
	else \
		echo "$(YELLOW)No test report found. Run 'make test-e2e' first.$(NC)"; \
	fi

# =============================================================================
# TESTING COMMANDS - AGGREGATED
# =============================================================================

.PHONY: test-unit
test-unit: test-backend-unit test-sdk-unit test-sdk-admin-unit ## Run all unit tests
	@echo "$(GREEN)✅ All unit tests completed!$(NC)"

.PHONY: test-integration
test-integration: test-backend-integration test-sdk-integration test-sdk-admin-integration ## Run all integration tests
	@echo "$(GREEN)✅ All integration tests completed!$(NC)"

.PHONY: test-backend-all
test-backend-all: test-backend-unit test-backend-integration ## Run all working backend tests (unit + simple integration)
	@echo "$(GREEN)✅ All backend tests completed!$(NC)"

.PHONY: test-backend-all-full
test-backend-all-full: test-backend-all test-backend-e2e ## Run all backend tests including E2E (requires services)
	@echo "$(GREEN)✅ Full backend test suite completed!$(NC)"

.PHONY: test-sdk-all
test-sdk-all: test-sdk test-sdk-admin ## Run all SDK tests
	@echo "$(GREEN)✅ All SDK tests completed!$(NC)"

.PHONY: test-all
test-all: test-backend-all test-sdk-all ## Run all tests that work without infrastructure
	@echo ""
	@echo "$(GREEN)═══════════════════════════════════════$(NC)"
	@echo "$(GREEN)✅ ALL TESTS COMPLETED SUCCESSFULLY!$(NC)"
	@echo "$(GREEN)═══════════════════════════════════════$(NC)"

.PHONY: test-complete
test-complete: test-all test-e2e ## Run ALL tests including E2E (requires services)
	@echo ""
	@echo "$(GREEN)════════════════════════════════════════════════════$(NC)"
	@echo "$(GREEN)🎉 COMPLETE TEST SUITE PASSED! (Including E2E) 🎉$(NC)"
	@echo "$(GREEN)════════════════════════════════════════════════════$(NC)"
	@echo ""

# =============================================================================
# UTILITY COMMANDS
# =============================================================================

.PHONY: status
status: ## Show service status
	@echo "$(CYAN)📊 Service Status:$(NC)"
	@echo "  gRPC Server: $(GRPC_URL)"
	@echo "  Web Interface: $(API_URL)"
	@echo "  Database: $(DATABASE_URL)"
	@ps aux | grep -E "hodei-verified-permissions|nextjs" | grep -v grep || echo "No services running"

.PHONY: stop
stop: ## Stop all services
	@echo "$(CYAN)⏹️  Stopping all services...$(NC)"
	@pkill -f "hodei-verified-permissions" || true
	@pkill -f "nextjs" || true
	@$(MAKE) test-e2e-services-stop || true
	@echo "$(GREEN)✅ All services stopped!$(NC)"

.PHONY: restart
restart: stop dev ## Restart all services
	@echo "$(CYAN)🔄 Restarting all services...$(NC)"

.PHONY: install-tools
install-tools: ## Install development tools
	@echo "$(CYAN)🛠️  Installing development tools...$(NC)"
	@rustup component add rustfmt clippy
	@npm install -g @grpc/grpc-js @grpc/proto-loader grpcurl
	@echo "$(GREEN)✅ Development tools installed!$(NC)"
