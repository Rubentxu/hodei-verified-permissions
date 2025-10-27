.PHONY: help build test test-unit test-integration test-e2e test-all clean docker-up docker-down docker-logs

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
	@echo "  $(YELLOW)make clean$(NC)              - Clean build artifacts"
	@echo ""
	@echo "$(GREEN)Test Targets:$(NC)"
	@echo "  $(YELLOW)make test$(NC)               - Run all tests (unit + integration)"
	@echo "  $(YELLOW)make test-unit$(NC)          - Run only unit tests (38 tests)"
	@echo "  $(YELLOW)make test-integration$(NC)   - Run only integration tests"
	@echo "  $(YELLOW)make test-e2e$(NC)           - Run E2E tests (requires Docker)"
	@echo "  $(YELLOW)make test-e2e-full$(NC)      - Run all E2E tests (requires Docker)"
	@echo "  $(YELLOW)make test-all$(NC)           - Run all tests (unit + integration + E2E)"
	@echo ""
	@echo "$(GREEN)Docker Targets:$(NC)"
	@echo "  $(YELLOW)make docker-up$(NC)          - Start Docker containers (SQLite profile)"
	@echo "  $(YELLOW)make docker-up-all$(NC)      - Start all Docker containers (all profiles)"
	@echo "  $(YELLOW)make docker-down$(NC)        - Stop Docker containers"
	@echo "  $(YELLOW)make docker-logs$(NC)        - Show Docker logs"
	@echo "  $(YELLOW)make docker-clean$(NC)       - Stop and remove Docker containers"
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

.PHONY: watch validate ci dev-setup
