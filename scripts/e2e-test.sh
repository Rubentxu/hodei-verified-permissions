#!/bin/bash

set -e

# ============================================================================
# E2E Test Automation Script
# Runs complete E2E tests for Hodei Verified Permissions
# ============================================================================

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_PORT=${RUST_PORT:-50051}
NEXTJS_PORT=${NEXTJS_PORT:-3000}
TEST_TIMEOUT=${TEST_TIMEOUT:-300} # 5 minutes
E2E_TEST_PATH=${E2E_TEST_PATH:-"web-nextjs/tests/e2e"}
PROJECT_ROOT="$SCRIPT_DIR/.."

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

# Test results
TEST_REPORT_DIR="$PROJECT_ROOT/test-results/e2e"
TEST_REPORT_FILE="$TEST_REPORT_DIR/results.html"

# ============================================================================
# Utility Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Note: Service management is handled by dev-start-managed.sh
# This script only handles testing concerns

# ============================================================================
# Setup Functions
# ============================================================================

install_playwright() {
    log_info "Installing Playwright browsers..."
    cd "$PROJECT_ROOT/web-nextjs"
    npx playwright install --with-deps
    cd "$PROJECT_ROOT"
    log_success "Playwright installed"
}

build_application() {
    log_info "Building application (libraries only)..."
    cd "$PROJECT_ROOT/verified-permissions"
    cargo build --lib
    cd "$PROJECT_ROOT"
    log_success "Application built"
}

wait_for_services() {
    log_info "Waiting for services to be ready..."

    local max_attempts=30
    local attempt=0

    # Wait for backend
    while [ $attempt -lt $max_attempts ]; do
        if curl -s http://localhost:$RUST_PORT/health > /dev/null 2>&1; then
            log_success "Backend is ready"
            break
        fi
        attempt=$((attempt + 1))
        log_info "Waiting for backend... ($attempt/$max_attempts)"
        sleep 2
    done

    if [ $attempt -eq $max_attempts ]; then
        log_error "Backend failed to start"
        return 1
    fi

    # Wait for frontend
    attempt=0
    while [ $attempt -lt $max_attempts ]; do
        if curl -s http://localhost:$NEXTJS_PORT/api/health > /dev/null 2>&1; then
            log_success "Frontend is ready"
            break
        fi
        attempt=$((attempt + 1))
        log_info "Waiting for frontend... ($attempt/$max_attempts)"
        sleep 2
    done

    if [ $attempt -eq $max_attempts ]; then
        log_error "Frontend failed to start"
        return 1
    fi

    log_success "All services are ready"
}

run_e2e_tests() {
    log_info "Running E2E tests..."

    # Create test results directory
    mkdir -p "$TEST_REPORT_DIR"

    # Run tests with Playwright
    cd "$PROJECT_ROOT/web-nextjs"

    # Set timeout for Playwright tests
    export PLAYWRIGHT_TIMEOUT=$((TEST_TIMEOUT * 1000))

    # Run tests with all reporters
    npx playwright test \
        --reporter=html,line \
        --output="$TEST_REPORT_DIR" \
        --report="$TEST_REPORT_FILE" \
        "$@"

    local test_exit_code=$?

    cd "$PROJECT_ROOT"

    if [ $test_exit_code -eq 0 ]; then
        log_success "All E2E tests passed"
        return 0
    else
        log_error "Some E2E tests failed"
        return 1
    fi
}

run_e2e_tests_ui() {
    log_info "Running E2E tests in UI mode..."
    cd "$PROJECT_ROOT/web-nextjs"
    npx playwright test --ui "$@"
    cd "$PROJECT_ROOT"
}

run_e2e_tests_headed() {
    log_info "Running E2E tests in headed mode..."
    cd "$PROJECT_ROOT/web-nextjs"
    npx playwright test --headed "$@"
    cd "$PROJECT_ROOT"
}

run_e2e_tests_debug() {
    log_info "Running E2E tests in debug mode..."
    cd "$PROJECT_ROOT/web-nextjs"
    npx playwright test --debug "$@"
    cd "$PROJECT_ROOT"
}

run_specific_test() {
    local test_file=$1
    if [ -z "$test_file" ]; then
        log_error "Test file not specified"
        return 1
    fi

    log_info "Running specific test: $test_file"
    run_e2e_tests "$E2E_TEST_PATH/$test_file"
}

# ============================================================================
# Main Functions
# ============================================================================

run_full_e2e_suite() {
    log_info "Starting full E2E test suite..."

    # Verify environment
    if [ ! -d "$PROJECT_ROOT/web-nextjs" ]; then
        log_error "Frontend directory not found"
        return 1
    fi

    if [ ! -d "$PROJECT_ROOT/verified-permissions" ]; then
        log_error "Backend directory not found"
        return 1
    fi

    # Install Playwright if needed
    if ! npx playwright --version > /dev/null 2>&1; then
        install_playwright
    fi

    # Build application
    build_application

    # Wait for services to be ready
    log_info "Waiting for services to be ready..."
    if ! wait_for_services; then
        log_error "Services are not running. Start them with: ./scripts/dev-start-managed.sh start-all"
        return 1
    fi

    # Run tests
    run_e2e_tests

    # Display results
    log_success "E2E test suite completed"
    log_info "Test report: $TEST_REPORT_FILE"
}

run_tests_only() {
    log_info "Running E2E tests (services must be running)..."

    # Wait for services to be ready
    if ! wait_for_services; then
        log_error "Services are not running. Start them with: ./scripts/dev-start-managed.sh start-all"
        return 1
    fi

    # Run tests
    run_e2e_tests "${@:2}"

    log_success "E2E tests completed"
    log_info "Test report: $TEST_REPORT_FILE"
}

show_help() {
    cat << EOF
E2E Test Automation Script

Usage: $0 [COMMAND] [OPTIONS]

Commands:
    full            Run full E2E test suite (builds + tests)
    install         Install Playwright browsers only
    build           Build application only
    test            Run tests only (requires running services)
    ui              Run tests with Playwright UI
    headed          Run tests in headed mode (visible browser)
    debug           Run tests in debug mode
    test-file       Run specific test file
    status          Show services status
    help            Show this help message

Options:
    --browser=CHROME|FIREFOX|WEBKIT    Run tests on specific browser
    --project=NAME                     Run specific test project
    --grep="PATTERN"                   Run tests matching pattern

Examples:
    $0 full                             # Run full E2E suite (builds + tests)
    $0 test                             # Run tests (services must be running)
    $0 ui                               # Run tests in UI mode
    $0 test-file policy-stores.spec.ts  # Run specific test file
    $0 ui --grep="PS-001"              # Run tests matching PS-001

Note: Service management (start/stop/status) is handled by dev-start-managed.sh

Environment Variables:
    RUST_PORT              Backend port (default: 50051)
    NEXTJS_PORT            Frontend port (default: 3000)
    TEST_TIMEOUT           Test timeout in seconds (default: 300)

EOF
}

# ============================================================================
# Main Logic
# ============================================================================

case "${1:-full}" in
    full)
        run_full_e2e_suite
        ;;
    install)
        install_playwright
        ;;
    build)
        build_application
        ;;
    test)
        run_tests_only "${@:2}"
        ;;
    ui)
        wait_for_services
        run_e2e_tests_ui "${@:2}"
        ;;
    headed)
        wait_for_services
        run_e2e_tests_headed "${@:2}"
        ;;
    debug)
        wait_for_services
        run_e2e_tests_debug "${@:2}"
        ;;
    test-file)
        wait_for_services
        run_specific_test "$2"
        ;;
    status)
        "$SCRIPT_DIR/dev-start-managed.sh" status
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        log_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
