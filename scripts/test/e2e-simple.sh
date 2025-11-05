#!/bin/bash
# =============================================================================
# Simple E2E Test Runner
# This script runs E2E tests by starting services directly (no Docker)
# =============================================================================

set -e

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

# Trap for cleanup on exit
cleanup() {
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        echo -e "\n${RED}❌ E2E tests failed with exit code $exit_code${NC}"
    fi

    # Cleanup background processes if we started them
    if [ -n "$BACKEND_PID" ] && kill -0 $BACKEND_PID 2>/dev/null; then
        echo -e "\n${YELLOW}🧹 Stopping backend server (PID: $BACKEND_PID)...${NC}"
        kill $BACKEND_PID 2>/dev/null || true
        wait $BACKEND_PID 2>/dev/null || true
    fi

    if [ -n "$FRONTEND_PID" ] && kill -0 $FRONTEND_PID 2>/dev/null; then
        echo -e "${YELLOW}🧹 Stopping frontend server (PID: $FRONTEND_PID)...${NC}"
        kill $FRONTEND_PID 2>/dev/null || true
        wait $FRONTEND_PID 2>/dev/null || true
    fi

    # Note: We use the shared database at /home/rubentxu/hodei-data/hodei.db
    # No cleanup needed as it's reused across runs

    exit $exit_code
}

trap cleanup EXIT INT TERM

# Function to check if a port is in use
check_port() {
    local port=$1
    if netstat -tlnp 2>/dev/null | grep -q ":$port "; then
        return 0
    else
        return 1
    fi
}

# Function to check if Next.js process is running
check_nextjs_process() {
    if ps aux | grep -E "next dev" | grep -v grep > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to find actual Next.js port
find_nextjs_port() {
    # Try to find Next.js running on ports 3000-3005
    for port in 3000 3001 3002 3003 3004 3005; do
        if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
            # Check if it's a Next.js process
            local pid=$(lsof -Pi :$port -sTCP:LISTEN -t 2>/dev/null)
            if ps -p $pid -o comm= 2>/dev/null | grep -E "node|next" > /dev/null; then
                echo "$port"
                return 0
            fi
        fi
    done
    echo "3000"  # Default fallback
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

# Ensure clean environment
echo -e "${YELLOW}🧹 Cleaning up any existing services...${NC}"
pkill -f "hodei-verified-permissions|nextjs" 2>/dev/null || true
sleep 2

# Install Playwright if needed
if ! npx playwright --version > /dev/null 2>&1; then
    echo -e "${YELLOW}📦 Installing Playwright browsers...${NC}"
    cd web-nextjs
    npx playwright install --with-deps
    cd "$PROJECT_ROOT"
fi

# Start backend server
echo -e "${YELLOW}🚀 Starting backend server...${NC}"
cd verified-permissions/main
# Use the same database path as the Makefile
mkdir -p /home/rubentxu/hodei-data
# Start server in background with SQLite database (same as Makefile default)
DATABASE_URL=sqlite:///home/rubentxu/hodei-data/hodei.db cargo run --bin hodei-verified-permissions > /tmp/backend.log 2>&1 &
BACKEND_PID=$!
cd "$PROJECT_ROOT"

# Wait for backend to be ready
wait_for_service 50051 "gRPC server" || exit 1

# Start frontend server
echo -e "${YELLOW}🚀 Starting frontend server...${NC}"
cd web-nextjs
# Start Next.js in background
npm run dev > /tmp/frontend.log 2>&1 &
FRONTEND_PID=$!
cd "$PROJECT_ROOT"

# Wait for frontend to be ready
wait_for_service 3000 "Next.js server" || exit 1

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
    echo -e "${YELLOW}📜 Backend logs: /tmp/backend.log${NC}"
    echo -e "${YELLOW}📜 Frontend logs: /tmp/frontend.log${NC}"
    exit 1
fi
