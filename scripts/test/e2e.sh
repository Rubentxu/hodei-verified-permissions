#!/bin/bash
# =============================================================================
# Run E2E Tests
# This script runs end-to-end tests using Playwright
# Usage: ./scripts/test/e2e.sh [command]
# Commands: install | test | ui | headed | debug | full (default: full)
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/../.."
COMMAND=${1:-full}

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

# Trap for cleanup on exit
cleanup() {
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        echo -e "\n${RED}❌ E2E tests failed with exit code $exit_code${NC}"
    fi
    # Only cleanup if we're in full mode and we started services
    if [ "$COMMAND" = "full" ] && [ -n "$INFRA_STARTED" ]; then
        echo -e "\n${YELLOW}🧹 Cleaning up infrastructure...${NC}"
        "$SCRIPT_DIR/infra-down.sh" sqlite 2>/dev/null || true
    fi
    exit $exit_code
}

trap cleanup EXIT

echo -e "${BLUE}🧪 E2E Test Runner${NC}"

cd "$PROJECT_ROOT"

# Function to check if a port is in use
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to wait for service to be ready
wait_for_service() {
    local port=$1
    local name=$2
    local max_attempts=30
    local attempt=0

    echo -e "${YELLOW}⏳ Waiting for $name on port $port...${NC}"

    while [ $attempt -lt $max_attempts ]; do
        if check_port $port; then
            echo -e "${GREEN}✅ $name is ready${NC}"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 2
    done

    echo -e "${RED}❌ $name failed to start on port $port${NC}"
    return 1
}

case $COMMAND in
    install)
        echo -e "${YELLOW}📦 Installing Playwright browsers...${NC}"
        cd web-nextjs
        npx playwright install --with-deps
        cd "$PROJECT_ROOT"
        echo -e "${GREEN}✅ Playwright installed${NC}"
        ;;
    test)
        echo -e "${YELLOW}🧪 Running E2E tests...${NC}"
        cd web-nextjs
        npx playwright test --reporter=line
        cd "$PROJECT_ROOT"
        echo -e "${GREEN}✅ E2E tests completed${NC}"
        ;;
    ui)
        echo -e "${YELLOW}🧪 Running E2E tests in UI mode...${NC}"
        cd web-nextjs
        npx playwright test --ui
        cd "$PROJECT_ROOT"
        ;;
    headed)
        echo -e "${YELLOW}🧪 Running E2E tests in headed mode...${NC}"
        cd web-nextjs
        npx playwright test --headed
        cd "$PROJECT_ROOT"
        ;;
    debug)
        echo -e "${YELLOW}🧪 Running E2E tests in debug mode...${NC}"
        cd web-nextjs
        npx playwright test --debug
        cd "$PROJECT_ROOT"
        ;;
    full)
        echo -e "${YELLOW}🚀 Running full E2E test suite...${NC}"

        # Install Playwright if needed
        if ! npx playwright --version > /dev/null 2>&1; then
            echo -e "${YELLOW}📦 Installing Playwright browsers...${NC}"
            cd web-nextjs
            npx playwright install --with-deps
            cd "$PROJECT_ROOT"
        fi

        # Check if services are already running
        INFRA_ALREADY_RUNNING=false
        if check_port 3000 && check_port 50051; then
            echo -e "${YELLOW}⚠️  Services already running on ports 3000 and 50051${NC}"
            echo -e "${YELLOW}   Will not start/stop infrastructure${NC}"
            INFRA_ALREADY_RUNNING=true
        fi

        # Start infrastructure only if not already running
        if [ "$INFRA_ALREADY_RUNNING" = false ]; then
            echo -e "${YELLOW}🚀 Starting infrastructure...${NC}"
            if "$SCRIPT_DIR/infra-up.sh" sqlite; then
                INFRA_STARTED=true
            else
                echo -e "${RED}❌ Failed to start infrastructure${NC}"
                exit 1
            fi

            # Wait for services to be ready
            wait_for_service 3000 "Next.js server" || exit 1
            wait_for_service 50051 "gRPC server" || exit 1

            # Give services a bit more time to stabilize
            echo -e "${YELLOW}⏳ Waiting for services to stabilize...${NC}"
            sleep 5
        fi

        # Run tests
        echo -e "${YELLOW}🧪 Running E2E tests...${NC}"
        cd web-nextjs

        # Run tests with both HTML and line reporters
        if npx playwright test --reporter=html,line; then
            cd "$PROJECT_ROOT"
            echo -e "${GREEN}✅ Full E2E test suite completed successfully${NC}"
        else
            cd "$PROJECT_ROOT"
            echo -e "${RED}❌ E2E tests failed${NC}"
            echo -e "${YELLOW}📊 Check the report at: web-nextjs/playwright-report/index.html${NC}"
            exit 1
        fi
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $COMMAND${NC}"
        echo "Valid commands: install, test, ui, headed, debug, full"
        exit 1
        ;;
esac
