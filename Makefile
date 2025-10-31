.PHONY: help build build-debug build-server build-server-debug test test-unit test-integration test-e2e test-e2e-full test-e2e-real test-e2e-real-dev test-all clean server-run server-run-debug kill-server kill-frontend kill-all docker-up docker-up-all docker-down docker-logs docker-clean docker-status fmt lint check doc info validate ci dev-setup watch bff-build bff-dev bff-test bff-health

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m # No Color

# Default target
help:
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Hodei Verified Permissions - Makefile                     ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "$(GREEN)Build Targets:$(NC)"
	@echo "  $(YELLOW)make build$(NC)              - Build all packages (release mode)"
	@echo "  $(YELLOW)make build-debug$(NC)        - Build all packages (debug mode)"
	@echo "  $(YELLOW)make build-server$(NC)       - Build verified-permissions server (release)"
	@echo "  $(YELLOW)make build-server-debug$(NC) - Build verified-permissions server (debug)"
	@echo "  $(YELLOW)make clean$(NC)              - Clean build artifacts"
	@echo ""
	@echo "$(GREEN)Test Targets:$(NC)"
	@echo "  $(YELLOW)make test$(NC)               - Run all tests (unit + integration)"
	@echo "  $(YELLOW)make test-unit$(NC)          - Run only unit tests (38 tests)"
	@echo "  $(YELLOW)make test-integration$(NC)   - Run only integration tests"
	@echo "  $(YELLOW)make test-e2e$(NC)           - Run E2E tests (requires Docker)"
	@echo "  $(YELLOW)make test-e2e-full$(NC)      - Run all E2E tests (requires Docker)"
	@echo "  $(YELLOW)make test-e2e-real$(NC)      - Run E2E tests against real server (release)"
	@echo "  $(YELLOW)make test-e2e-real-dev$(NC)  - Run E2E tests against real server (debug)"
	@echo "  $(YELLOW)make test-all$(NC)           - Run all tests (unit + integration + E2E)"
	@echo ""
	@echo "$(GREEN)Server Targets:$(NC)"
	@echo "  $(YELLOW)make server-run$(NC)         - Run server (release mode)"
	@echo "  $(YELLOW)make server-run-debug$(NC)   - Run server (debug mode)"
	@echo ""
	@echo "$(GREEN)Process Management:$(NC)"
	@echo "  $(YELLOW)make kill-server$(NC)        - Kill all server processes"
	@echo "  $(YELLOW)make kill-frontend$(NC)      - Kill all frontend processes"
	@echo "  $(YELLOW)make kill-all$(NC)           - Kill all server and frontend processes"
	@echo ""
	@echo "$(GREEN)Docker Targets:$(NC)"
	@echo "  $(YELLOW)make docker-up$(NC)          - Start Docker containers (SQLite profile)"
	@echo "  $(YELLOW)make docker-up-all$(NC)      - Start all Docker containers (all profiles)"
	@echo "  $(YELLOW)make docker-down$(NC)        - Stop Docker containers"
	@echo "  $(YELLOW)make docker-logs$(NC)        - Show Docker logs"
	@echo "  $(YELLOW)make docker-clean$(NC)       - Stop and remove Docker containers"
	@echo ""
	@echo "$(GREEN)BFF (Backend for Frontend) Targets:$(NC)"
	@echo "  $(YELLOW)make bff-build$(NC)          - Build Next.js BFF for production"
	@echo "  $(YELLOW)make bff-dev$(NC)            - Start Next.js BFF in development mode"
	@echo "  $(YELLOW)make bff-test$(NC)           - Test BFF gRPC connectivity"
	@echo "  $(YELLOW)make bff-health$(NC)         - Check BFF and gRPC backend health"
	@echo ""
	@echo "$(GREEN)Development Targets:$(NC)"
	@echo "  $(YELLOW)make fmt$(NC)                - Format code (rustfmt)"
	@echo "  $(YELLOW)make lint$(NC)               - Run clippy linter"
	@echo "  $(YELLOW)make check$(NC)              - Check code without building"
	@echo "  $(YELLOW)make doc$(NC)                - Generate documentation"
	@echo ""
	@echo "$(GREEN)Utility Targets:$(NC)"
	@echo "  $(YELLOW)make info$(NC)               - Show project info"
	@echo "  $(YELLOW)make help$(NC)               - Show this help message"
	@echo ""

# ============================================================================
# BUILD TARGETS
# ============================================================================

build:
	@echo "$(GREEN)Building all packages (release mode)...$(NC)"
	cargo build --all --release
	@echo "$(GREEN)✅ Build completed successfully$(NC)"

build-debug:
	@echo "$(GREEN)Building all packages (debug mode)...$(NC)"
	cargo build --all
	@echo "$(GREEN)✅ Build completed successfully$(NC)"

build-server:
	@echo "$(GREEN)Building verified-permissions server (release mode)...$(NC)"
	cd verified-permissions && cargo build --release
	@echo "$(GREEN)✅ Server built: ./verified-permissions/target/release/hodei-verified-permissions$(NC)"

build-server-debug:
	@echo "$(GREEN)Building verified-permissions server (debug mode)...$(NC)"
	cd verified-permissions && cargo build
	@echo "$(GREEN)✅ Server built: ./verified-permissions/target/debug/hodei-verified-permissions$(NC)"

clean:
	@echo "$(YELLOW)Cleaning build artifacts...$(NC)"
	cargo clean
	@echo "$(GREEN)✅ Clean completed$(NC)"

# ============================================================================
# TEST TARGETS
# ============================================================================

test: test-unit test-integration
	@echo "$(GREEN)✅ All tests passed!$(NC)"

test-unit:
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Running Unit Tests (38 tests)                             ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	cargo test --lib --all
	@echo "$(GREEN)✅ Unit tests passed!$(NC)"

test-integration:
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Running Integration Tests                                 ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	cargo test --test '*' --all
	@echo "$(GREEN)✅ Integration tests completed!$(NC)"

test-e2e: docker-up
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Running E2E Tests (Full Stack)                            ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	@echo "$(YELLOW)Waiting for services to be ready...$(NC)"
	sleep 15
	cargo test --test e2e_full_stack -- --ignored --nocapture
	@echo "$(GREEN)✅ E2E tests completed!$(NC)"

test-e2e-full: docker-up-all
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Running All E2E Tests                                     ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	@echo "$(YELLOW)Waiting for services to be ready...$(NC)"
	sleep 30
	cargo test -- --ignored --nocapture
	@echo "$(GREEN)✅ All E2E tests completed!$(NC)"

test-all: test docker-up
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Running All Tests (Unit + Integration + E2E)              ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	@echo "$(YELLOW)Waiting for services to be ready...$(NC)"
	sleep 15
	cargo test --all -- --ignored --nocapture
	@echo "$(GREEN)✅ All tests completed!$(NC)"

# ============================================================================
# SERVER TARGETS (Real Server Execution)
# ============================================================================

server-run: build-server
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Starting Hodei Verified Permissions Server                ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	@echo "$(YELLOW)Killing any existing server processes...$(NC)"
	@pkill -9 -f "hodei-verified-permissions" 2>/dev/null || true
	@sleep 1
	@echo "$(YELLOW)Server listening on: 0.0.0.0:50051$(NC)"
	@echo "$(YELLOW)Press Ctrl+C to stop$(NC)"
	@mkdir -p /home/rubentxu/hodei-data
	@DATABASE_URL="sqlite:////home/rubentxu/hodei-data/hodei.db" ./verified-permissions/target/release/hodei-verified-permissions

server-run-debug: build-server-debug
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Starting Hodei Verified Permissions Server (Debug)        ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	@echo "$(YELLOW)Killing any existing server processes...$(NC)"
	@pkill -9 -f "hodei-verified-permissions" 2>/dev/null || true
	@sleep 1
	@echo "$(YELLOW)Server listening on: 0.0.0.0:50051$(NC)"
	@echo "$(YELLOW)Press Ctrl+C to stop$(NC)"
	@mkdir -p /home/rubentxu/hodei-data
	@DATABASE_URL="sqlite:////home/rubentxu/hodei-data/hodei.db" ./verified-permissions/target/debug/hodei-verified-permissions

# ============================================================================
# E2E TESTS WITH REAL SERVER
# ============================================================================

test-e2e-real: build-server
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Running E2E Tests Against Real Server (Release)           ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	@./scripts/run-e2e-tests.sh release

test-e2e-real-dev: build-server-debug
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Running E2E Tests Against Real Server (Debug)             ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	@./scripts/run-e2e-tests.sh debug

# ============================================================================
# PROCESS MANAGEMENT TARGETS
# ============================================================================

kill-server:
	@echo "$(YELLOW)Killing server processes...$(NC)"
	@pkill -9 -f "hodei-verified-permissions" 2>/dev/null || echo "No server processes found"
	@sleep 1
	@echo "$(GREEN)✅ Server processes killed$(NC)"

kill-frontend:
	@echo "$(YELLOW)Killing frontend processes...$(NC)"
	@pkill -9 -f "next dev" 2>/dev/null || echo "No frontend processes found"
	@pkill -9 -f "next start" 2>/dev/null || echo "No frontend processes found"
	@sleep 1
	@echo "$(GREEN)✅ Frontend processes killed$(NC)"

kill-all: kill-server kill-frontend
	@echo "$(GREEN)✅ All processes killed$(NC)"

# ============================================================================
# DOCKER TARGETS
# ============================================================================

docker-up:
	@echo "$(GREEN)Starting Docker containers (SQLite profile)...$(NC)"
	docker compose -f docker-compose.test.yml --profile sqlite up -d
	@echo "$(GREEN)✅ Docker containers started$(NC)"
	@echo "$(YELLOW)Services:$(NC)"
	@echo "  - Hodei Server: http://localhost:50051"
	@echo "  - TODO App: http://localhost:3000"

docker-up-all:
	@echo "$(GREEN)Starting all Docker containers (all profiles)...$(NC)"
	docker compose -f docker-compose.test.yml --profile all up -d
	@echo "$(GREEN)✅ Docker containers started$(NC)"
	@echo "$(YELLOW)Services:$(NC)"
	@echo "  - Hodei Server (SQLite): http://localhost:50051"
	@echo "  - Hodei Server (PostgreSQL): http://localhost:50052"
	@echo "  - Hodei Server (SurrealDB): http://localhost:50053"
	@echo "  - TODO App (SQLite): http://localhost:3000"
	@echo "  - TODO App (PostgreSQL): http://localhost:3001"
	@echo "  - TODO App (SurrealDB): http://localhost:3002"
	@echo "  - PostgreSQL: localhost:5432"
	@echo "  - SurrealDB: localhost:8001"

docker-down:
	@echo "$(YELLOW)Stopping Docker containers...$(NC)"
	docker compose -f docker-compose.test.yml down
	@echo "$(GREEN)✅ Docker containers stopped$(NC)"

docker-logs:
	@echo "$(YELLOW)Showing Docker logs...$(NC)"
	docker compose -f docker-compose.test.yml logs -f

docker-clean:
	@echo "$(YELLOW)Stopping and removing Docker containers...$(NC)"
	docker compose -f docker-compose.test.yml down -v
	@echo "$(GREEN)✅ Docker containers removed$(NC)"

docker-status:
	@echo "$(BLUE)Docker Container Status:$(NC)"
	docker compose -f docker-compose.test.yml ps

# ============================================================================
# DEVELOPMENT TARGETS
# ============================================================================

fmt:
	@echo "$(GREEN)Formatting code...$(NC)"
	cargo fmt --all
	@echo "$(GREEN)✅ Code formatted$(NC)"

lint:
	@echo "$(GREEN)Running clippy linter...$(NC)"
	cargo clippy --all --all-targets -- -D warnings
	@echo "$(GREEN)✅ Linting completed$(NC)"

check:
	@echo "$(GREEN)Checking code...$(NC)"
	cargo check --all
	@echo "$(GREEN)✅ Check completed$(NC)"

doc:
	@echo "$(GREEN)Generating documentation...$(NC)"
	cargo doc --all --no-deps --open
	@echo "$(GREEN)✅ Documentation generated$(NC)"

# ============================================================================
# UTILITY TARGETS
# ============================================================================

info:
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Project Information                                       ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	@echo ""
	@echo "$(GREEN)Project:$(NC) Hodei Verified Permissions"
	@echo "$(GREEN)Language:$(NC) Rust"
	@echo ""
	@echo "$(GREEN)Packages:$(NC)"
	@echo "  - hodei-verified-permissions (main server)"
	@echo "  - hodei-permissions-sdk (client SDK)"
	@echo "  - hodei-macros (procedural macros)"
	@echo "  - hodei-cli (command line interface)"
	@echo ""
	@echo "$(GREEN)Test Summary:$(NC)"
	@echo "  - Unit Tests: 38 tests"
	@echo "  - Integration Tests: Available"
	@echo "  - E2E Tests: 47 tests (requires Docker)"
	@echo ""
	@echo "$(GREEN)Rust Version:$(NC)"
	@rustc --version
	@echo "$(GREEN)Cargo Version:$(NC)"
	@cargo --version
	@echo ""

# ============================================================================
# QUICK COMMANDS
# ============================================================================

# Run quick validation (check + lint + unit tests)
validate: check lint test-unit
	@echo "$(GREEN)✅ Validation passed!$(NC)"

# Run full CI pipeline
ci: build fmt lint test-unit
	@echo "$(GREEN)✅ CI pipeline completed!$(NC)"

# Development setup
dev-setup: build fmt lint
	@echo "$(GREEN)✅ Development environment ready!$(NC)"

# Watch for changes and run tests
watch:
	@echo "$(GREEN)Watching for changes...$(NC)"
	cargo watch -x "test --lib" -x "clippy"

# ============================================================================
# BFF (BACKEND FOR FRONTEND) TARGETS
# ============================================================================

# Build Next.js BFF for production
bff-build:
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Building Next.js BFF for Production                        ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	cd web-nextjs && npm install && npm run build

# Start Next.js BFF in development mode
bff-dev:
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Starting Next.js BFF in Development Mode                   ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	cd web-nextjs && npm install && npm run dev

# Test BFF gRPC connectivity (requires server running)
bff-test:
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Testing BFF gRPC Connectivity                               ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	@echo "$(YELLOW)Note: Make sure the Rust server is running on localhost:50051$(NC)"
	cd web-nextjs && npm run build && npm start &
	@sleep 5
	@curl -s -X POST http://localhost:3000/api/authorize \
		-H "Content-Type: application/json" \
		-d '{"policy_store_id":"test","principal":{"entity_type":"User","entity_id":"alice"},"action":{"entity_type":"Action","entity_id":"view"},"resource":{"entity_type":"Document","entity_id":"doc1"}}' \
		| jq . || echo "Test completed"
	@pkill -f "npm start" 2>/dev/null || true

# Check BFF and gRPC backend health
bff-health:
	@echo "$(BLUE)╔════════════════════════════════════════════════════════════╗$(NC)"
	@echo "$(BLUE)║  Checking BFF and gRPC Backend Health                        ║$(NC)"
	@echo "$(BLUE)╚════════════════════════════════════════════════════════════╝$(NC)"
	@echo "$(YELLOW)Checking gRPC server health...$(NC)"
	@curl -s http://localhost:50051/health || echo "gRPC server not responding on localhost:50051"
	@echo ""
	@echo "$(YELLOW)Checking BFF health...$(NC)"
	cd web-nextjs && npm run build && npm start > /tmp/bff-health.log 2>&1 &
	@sleep 5
	@curl -s http://localhost:3000/api/health | jq . || echo "BFF not responding"
	@pkill -f "npm start" 2>/dev/null || true

# Development targets for integrated server and frontend
dev-start:
	@bash ./scripts/dev-start.sh

dev-logs:
	@tail -f /tmp/rust-server.log

dev-logs-frontend:
	@tail -f /tmp/nextjs-server.log

dev-test:
	@echo "$(BLUE)Testing gRPC connection...$(NC)"
	@sleep 2
	@curl -s http://localhost:3000/api/health | jq . || echo "Health check failed"

dev-stop: kill-all
	@echo "$(GREEN)Development environment stopped$(NC)"

.PHONY: watch validate ci dev-setup bff-build bff-dev bff-test bff-health dev-start dev-logs dev-logs-frontend dev-test dev-stop
