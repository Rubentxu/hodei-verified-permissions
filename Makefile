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
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""

# =============================================================================
# Development
# =============================================================================

.PHONY: dev
dev: ## Start all services in development mode
	@echo "$(CYAN)🚀 Starting development environment...$(NC)"
	@$(MAKE) build
	@$(MAKE) db-init
	@$(MAKE) server &
	@$(MAKE) web &
	@echo "$(GREEN)✅ Services started!$(NC)"
	@echo "$(YELLOW)📍 Frontend: http://localhost:3000$(NC)"
	@echo "$(YELLOW)📍 gRPC API: localhost:50051$(NC)"

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
# Database
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

.PHONY: db-migrate
db-migrate: ## Run database migrations
	@echo "$(CYAN)📊 Running migrations...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo run --bin hodei-verified-permissions -- --migrate
	@echo "$(GREEN)✅ Migrations completed!$(NC)"

# =============================================================================
# Server Management
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

.PHONY: server-logs
server-logs: ## Show server logs (if running in background)
	@echo "$(CYAN)📋 Server logs...$(NC)"
	@journalctl -u hodei-verified-permissions -f 2>/dev/null || echo "Service not running as systemd service"

# =============================================================================
# Web Interface
# =============================================================================

.PHONY: web
web: ## Start Next.js web interface
	@echo "$(CYAN)🌐 Starting web interface...$(NC)"
	@cd $(PROJECT_ROOT)/web-nextjs && \
	npm run dev

.PHONY: web-build
web-build: ## Build Next.js for production
	@echo "$(CYAN)🔨 Building web interface...$(NC)"
	@cd $(PROJECT_ROOT)/web-nextjs && npm run build
	@echo "$(GREEN)✅ Web build completed!$(NC)"

.PHONY: web-start
web-start: web-build ## Start Next.js in production mode
	@echo "$(CYAN)🌐 Starting web interface (production)...$(NC)"
	@cd $(PROJECT_ROOT)/web-nextjs && npm start

# =============================================================================
# Testing
# =============================================================================

.PHONY: test
test: ## Run all tests (unit + integration)
	@echo "$(CYAN)🧪 Running all tests...$(NC)"
	@$(MAKE) test-unit
	@$(MAKE) test-integration

.PHONY: test-unit
test-unit: ## Run unit tests
	@echo "$(CYAN)🧪 Running unit tests...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo test --lib
	@echo "$(GREEN)✅ Unit tests completed!$(NC)"

.PHONY: test-integration
test-integration: ## Run integration tests
	@echo "$(CYAN)🧪 Running integration tests...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo test --test '*'
	@echo "$(GREEN)✅ Integration tests completed!$(NC)"

.PHONY: test-all
test-all: ## Run all tests with coverage
	@echo "$(CYAN)🧪 Running all tests with coverage...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo test --all --coverage
	@echo "$(GREEN)✅ All tests with coverage completed!$(NC)"

.PHONY: test-watch
test-watch: ## Run tests in watch mode
	@echo "$(CYAN)👀 Running tests in watch mode...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo watch -x test

.PHONY: benchmark
benchmark: ## Run benchmarks
	@echo "$(CYAN)⚡ Running benchmarks...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo bench
	@echo "$(GREEN)✅ Benchmarks completed!$(NC)"

# =============================================================================
# Code Quality
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
# gRPC Testing
# =============================================================================

.PHONY: grpc-reflect
grpc-reflect: ## List available gRPC services
	@echo "$(CYAN)📋 Available gRPC services...$(NC)"
	@grpcurl -plaintext $(GRPC_URL) describe

.PHONY: grpc-test
grpc-test: ## Run basic gRPC test
	@echo "$(CYAN)🧪 Testing gRPC connection...$(NC)"
	@grpcurl -plaintext $(GRPC_URL) list

.PHONY: grpc-health
grpc-health: ## Check gRPC service health
	@echo "$(CYAN)💚 Checking gRPC service health...$(NC)"
	@grpcurl -plaintext -d '{}' $(GRPC_URL) grpc.health.v1.Health.Check

# =============================================================================
# API Documentation
# =============================================================================

.PHONY: docs
docs: ## Generate API documentation
	@echo "$(CYAN)📚 Generating documentation...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo doc --no-deps --open
	@echo "$(GREEN)✅ Documentation generated!$(NC)"

.PHONY: docs-serve
docs-serve: ## Serve documentation locally
	@echo "$(CYAN)🌐 Serving documentation...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && cargo doc --no-deps --watch

# =============================================================================
# Docker
# =============================================================================

.PHONY: docker-build
docker-build: ## Build Docker image
	@echo "$(CYAN)🐳 Building Docker image...$(NC)"
	@cd $(PROJECT_ROOT)/verified-permissions && docker build -t hodei-verified-permissions .
	@echo "$(GREEN)✅ Docker image built!$(NC)"

.PHONY: docker-run
docker-run: ## Run Docker container
	@echo "$(CYAN)🐳 Running Docker container...$(NC)"
	@docker run -p 50051:50051 -p 3000:3000 hodei-verified-permissions
	@echo "$(GREEN)✅ Docker container running!$(NC)"

# =============================================================================
# Development Tools
# =============================================================================

.PHONY: install-tools
install-tools: ## Install development tools
	@echo "$(CYAN)🛠️  Installing development tools...$(NC)"
	@rustup component add rustfmt clippy
	@npm install -g @grpc/grpc-js @grpc/proto-loader grpcurl
	@echo "$(GREEN)✅ Development tools installed!$(NC)"

.PHONY: proto-generate
proto-generate: ## Generate protobuf files
	@echo "$(CYAN)📜 Generating protobuf files...$(NC)"
	@protoc --proto_path=$(PROJECT_ROOT)/proto \
		--rust_out=$(PROJECT_ROOT)/verified-permissions/api/src \
		--grpc-rust_out=$(PROJECT_ROOT)/verified-permissions/api/src \
		$(PROJECT_ROOT)/proto/*.proto
	@echo "$(GREEN)✅ Protobuf files generated!$(NC)"

# =============================================================================
# Monitoring
# =============================================================================

.PHONY: metrics
metrics: ## Start metrics collection
	@echo "$(CYAN)📊 Starting metrics collection...$(NC)"
	@curl -s $(API_URL)/api/metrics | jq '.'

.PHONY: health
health: ## Check service health
	@echo "$(CYAN)💚 Checking service health...$(NC)"
	@curl -s $(API_URL)/api/health | jq '.'

# =============================================================================
# Utility
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
	@echo "$(GREEN)✅ All services stopped!$(NC)"

.PHONY: restart
restart: stop dev ## Restart all services
	@echo "$(CYAN)🔄 Restarting all services...$(NC)"

# =============================================================================
# Postman Collection
# =============================================================================

.PHONY: postman-export
postman-export: ## Export Postman collection
	@echo "$(CYAN)📤 Exporting Postman collection...$(NC)"
	@cp $(PROJECT_ROOT)/docs/postman/VerifiedPermissions.postman_collection.json $(PROJECT_ROOT)/postman/
	@echo "$(GREEN)✅ Postman collection exported!$(NC)"

.PHONY: postman-import
postman-import: ## Import Postman collection to environment
	@echo "$(CYAN)📥 Importing Postman collection...$(NC)"
	@echo "Open Postman and import: $(PROJECT_ROOT)/postman/VerifiedPermissions.postman_collection.json"
	@echo "$(GREEN)✅ Ready to import!$(NC)"

# =============================================================================
# End-to-End Testing (E2E)
# =============================================================================

.PHONY: test-e2e
test-e2e: ## Run full E2E test suite (start services + run tests)
	@echo "$(CYAN)🚀 Starting services for E2E tests...$(NC)"
	@$(PROJECT_ROOT)/scripts/dev-start-managed.sh start-all
	@echo "$(CYAN)🧪 Running E2E test suite...$(NC)"
	@$(PROJECT_ROOT)/scripts/e2e-test.sh test
	@echo "$(GREEN)✅ E2E tests completed!$(NC)"

.PHONY: test-e2e-install
test-e2e-install: ## Install Playwright browsers
	@echo "$(CYAN)🌐 Installing Playwright browsers...$(NC)"
	@cd $(PROJECT_ROOT)/web-nextjs && npx playwright install --with-deps
	@echo "$(GREEN)✅ Playwright browsers installed!$(NC)"

.PHONY: test-e2e-start
test-e2e-start: ## Start services for E2E testing
	@echo "$(CYAN)🚀 Starting services for E2E tests...$(NC)"
	@$(PROJECT_ROOT)/scripts/dev-start-managed.sh start-all
	@echo "$(GREEN)✅ Services started!$(NC)"

.PHONY: test-e2e-ui
test-e2e-ui: ## Run E2E tests with Playwright UI (requires running services)
	@echo "$(CYAN)🧪 Running E2E tests in UI mode...$(NC)"
	@$(PROJECT_ROOT)/scripts/e2e-test.sh ui
	@echo "$(GREEN)✅ E2E tests completed!$(NC)"

.PHONY: test-e2e-headed
test-e2e-headed: ## Run E2E tests in headed mode (visible browser, requires running services)
	@echo "$(CYAN)🧪 Running E2E tests in headed mode...$(NC)"
	@$(PROJECT_ROOT)/scripts/e2e-test.sh headed
	@echo "$(GREEN)✅ E2E tests completed!$(NC)"

.PHONY: test-e2e-debug
test-e2e-debug: ## Run E2E tests in debug mode (requires running services)
	@echo "$(CYAN)🧪 Running E2E tests in debug mode...$(NC)"
	@$(PROJECT_ROOT)/scripts/e2e-test.sh debug
	@echo "$(GREEN)✅ E2E tests completed!$(NC)"

.PHONY: test-e2e-policy-stores
test-e2e-policy-stores: ## Run Policy Store E2E tests (start services + run tests)
	@echo "$(CYAN)🚀 Starting services for Policy Store tests...$(NC)"
	@$(PROJECT_ROOT)/scripts/dev-start-managed.sh start-all
	@echo "$(CYAN)🧪 Running Policy Store E2E tests...$(NC)"
	@$(PROJECT_ROOT)/scripts/e2e-test.sh test-file policy-stores.spec.ts
	@echo "$(GREEN)✅ Policy Store E2E tests completed!$(NC)"

.PHONY: test-e2e-playground
test-e2e-playground: ## Run Playground E2E tests (start services + run tests)
	@echo "$(CYAN)🚀 Starting services for Playground tests...$(NC)"
	@$(PROJECT_ROOT)/scripts/dev-start-managed.sh start-all
	@echo "$(CYAN)🧪 Running Playground E2E tests...$(NC)"
	@$(PROJECT_ROOT)/scripts/e2e-test.sh test-file playground.spec.ts
	@echo "$(GREEN)✅ Playground E2E tests completed!$(NC)"

.PHONY: test-e2e-dashboard
test-e2e-dashboard: ## Run Dashboard E2E tests (start services + run tests)
	@echo "$(CYAN)🚀 Starting services for Dashboard tests...$(NC)"
	@$(PROJECT_ROOT)/scripts/dev-start-managed.sh start-all
	@echo "$(CYAN)🧪 Running Dashboard E2E tests...$(NC)"
	@$(PROJECT_ROOT)/scripts/e2e-test.sh test-file dashboard.spec.ts
	@echo "$(GREEN)✅ Dashboard E2E tests completed!$(NC)"

.PHONY: test-e2e-browser-chrome
test-e2e-browser-chrome: ## Run E2E tests on Chrome
	@echo "$(CYAN)🧪 Running E2E tests on Chrome...$(NC)"
	@$(PROJECT_ROOT)/scripts/e2e-test.sh test --browser=chrome
	@echo "$(GREEN)✅ Chrome E2E tests completed!$(NC)"

.PHONY: test-e2e-browser-firefox
test-e2e-browser-firefox: ## Run E2E tests on Firefox
	@echo "$(CYAN)🧪 Running E2E tests on Firefox...$(NC)"
	@$(PROJECT_ROOT)/scripts/e2e-test.sh test --browser=firefox
	@echo "$(GREEN)✅ Firefox E2E tests completed!$(NC)"

.PHONY: test-e2e-browser-webkit
test-e2e-browser-webkit: ## Run E2E tests on WebKit
	@echo "$(CYAN)🧪 Running E2E tests on WebKit...$(NC)"
	@$(PROJECT_ROOT)/scripts/e2e-test.sh test --browser=webkit
	@echo "$(GREEN)✅ WebKit E2E tests completed!$(NC)"

.PHONY: test-e2e-status
test-e2e-status: ## Check E2E test services status
	@echo "$(CYAN)📊 Checking E2E test services status...$(NC)"
	@$(PROJECT_ROOT)/scripts/e2e-test.sh status
	@echo "$(GREEN)✅ Status checked!$(NC)"

.PHONY: test-e2e-stop
test-e2e-stop: ## Stop E2E test services
	@echo "$(CYAN)⏹️  Stopping E2E test services...$(NC)"
	@$(PROJECT_ROOT)/scripts/dev-start-managed.sh stop-all
	@echo "$(GREEN)✅ Services stopped!$(NC)"

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
