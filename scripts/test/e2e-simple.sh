#!/bin/bash
# =============================================================================
# Simple E2E Test Runner
# This script runs E2E tests by using Makefile targets to manage services
# =============================================================================

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/../.."
cd "$PROJECT_ROOT"

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🧪 Simple E2E Test Runner${NC}"
echo -e "${YELLOW}Using Makefile targets for service management${NC}"

# Trap for cleanup on exit
cleanup() {
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        echo -e "\n${RED}❌ E2E tests failed with exit code $exit_code${NC}"
    fi

    # Use Makefile to stop services
    echo -e "\n${YELLOW}🧹 Stopping services...${NC}"
    make stop > /dev/null 2>&1 || true

    exit $exit_code
}

trap cleanup EXIT INT TERM

# Function to check if a port is in use
check_port() {
    local port=$1
    # Check using multiple methods to handle both IPv4 and IPv6
    lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1 && return 0
    netstat -tlnp 2>/dev/null | grep -q ":$port " && return 0
    ss -tlnp 2>/dev/null | grep -q ":$port " && return 0
    return 1
}

# Function to wait for service to be ready
wait_for_service() {
    local port=$1
    local name=$2
    local max_attempts=30
    local attempt=0

    echo -e "${YELLOW}⏳ Waiting for $name on port $port...${NC}"

    while [ $attempt -lt $max_attempts ]; do
        echo "[$(date +%T)] Attempt $attempt/$max_attempts - checking port $port..."
        if check_port $port; then
            echo -e "${GREEN}✅ $name is ready at $(date +%T)${NC}"
            return 0
        fi
        attempt=$((attempt + 1))
        echo "Port not ready yet, sleeping for 2 seconds..."
        sleep 2
    done

    echo -e "${RED}❌ $name failed to start on port $port${NC}"
    return 1
}

# Ensure clean environment and install Playwright
echo -e "${YELLOW}🧹 Cleaning up any existing services...${NC}"
make stop > /dev/null 2>&1 || true
sleep 2

echo -e "${YELLOW}📦 Checking Playwright installation...${NC}"
cd web-nextjs
if ! npx playwright --version > /dev/null 2>&1; then
    echo -e "${YELLOW}📦 Installing Playwright browsers...${NC}"
    npx playwright install --with-deps
fi
cd "$PROJECT_ROOT"

# Start all services using Makefile
echo -e "${YELLOW}🚀 Starting services with make dev...${NC}"
make dev > /tmp/services.log 2>&1 &

# Wait for backend to be ready
wait_for_service 50051 "gRPC server" || {
    echo -e "${RED}❌ Backend failed to start${NC}"
    echo -e "${YELLOW}Services log:${NC}"
    cat /tmp/services.log
    exit 1
}

# Wait for frontend to be ready
wait_for_service 3000 "Next.js server" || {
    echo -e "${RED}❌ Frontend failed to start${NC}"
    echo -e "${YELLOW}Services log:${NC}"
    cat /tmp/services.log
    exit 1
}

# Run tests
echo -e "${YELLOW}🧪 Running E2E tests...${NC}"
cd web-nextjs

# Run tests with HTML and line reporters
if npx playwright test --reporter=html,line; then
    cd "$PROJECT_ROOT"
    echo -e "${GREEN}✅ E2E tests completed successfully${NC}"
else
    cd "$PROJECT_ROOT"
    echo -e "${RED}❌ E2E tests failed${NC}"
    echo -e "${YELLOW}📊 Check the report at: web-nextjs/playwright-report/index.html${NC}"
    exit 1
fi
