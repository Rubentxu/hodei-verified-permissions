#!/bin/bash

set -e

# ============================================================================
# Configuration (can be overridden by environment variables)
# ============================================================================
RUST_PORT=${RUST_PORT:-50051}
NEXTJS_PORT=${NEXTJS_PORT:-3000}
LOG_DIR=${LOG_DIR:-/tmp}
DATA_DIR=${DATA_DIR:-./data}
MAX_RETRIES=${MAX_RETRIES:-3}
HEALTH_CHECK_RETRIES=${HEALTH_CHECK_RETRIES:-30}

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Starting Development Environment (Improved)                ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo -e "${YELLOW}Configuration:${NC}"
echo -e "  Rust Port:    ${BLUE}$RUST_PORT${NC}"
echo -e "  Next.js Port: ${BLUE}$NEXTJS_PORT${NC}"
echo -e "  Data Dir:     ${BLUE}$DATA_DIR${NC}"
echo -e "  Log Dir:      ${BLUE}$LOG_DIR${NC}"
echo ""

# ============================================================================
# Utility Functions
# ============================================================================

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Shutting down development environment...${NC}"
    pkill -f "hodei-verified-permissions" 2>/dev/null || true
    pkill -f "next dev" 2>/dev/null || true
    pkill -f "next" 2>/dev/null || true
    sleep 1
    echo -e "${GREEN}✅ Development environment stopped${NC}"
}

# Trap to ensure cleanup on exit (only for INT and TERM, not EXIT to avoid double execution)
trap cleanup INT TERM

# Check if port is in use
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0  # Port is in use
    fi
    return 1  # Port is free
}

# Wait for port to be ready
wait_for_port() {
    local port=$1
    local service_name=$2
    local max_attempts=$HEALTH_CHECK_RETRIES

    echo -e "${YELLOW}Waiting for $service_name to be ready on port $port...${NC}"

    for i in $(seq 1 $max_attempts); do
        if check_port $port; then
            echo -e "${GREEN}✓ $service_name is ready (attempt $i/$max_attempts)${NC}"
            return 0
        fi
        sleep 1
    done

    echo -e "${RED}✗ $service_NAME failed to start on port $port after $max_attempts seconds${NC}"
    return 1
}

# Start service with retry
start_with_retry() {
    local service_name=$1
    local start_command=$2
    local max_attempts=$MAX_RETRIES

    for i in $(seq 1 $max_attempts); do
        echo -e "${YELLOW}Starting $service_name (attempt $i/$max_attempts)...${NC}"

        if eval "$start_command"; then
            echo -e "${GREEN}✓ $service_name started successfully${NC}"
            return 0
        fi

        if [ $i -eq $max_attempts ]; then
            echo -e "${RED}✗ $service_name failed to start after $max_attempts attempts${NC}"
            return 1
        fi

        echo -e "${YELLOW}Retrying in 2 seconds...${NC}"
        sleep 2
    done
}

# ============================================================================
# Main Startup Logic
# ============================================================================

# Kill any existing processes (with delay to free ports)
echo -e "${YELLOW}Cleaning up existing processes...${NC}"
pkill -f "hodei-verified-permissions" 2>/dev/null || true
pkill -f "next dev" 2>/dev/null || true
pkill -f "next" 2>/dev/null || true
sleep 3

# Check for port conflicts after cleanup
echo -e "${YELLOW}Checking for port conflicts...${NC}"
if check_port $RUST_PORT; then
    echo -e "${RED}✗ Port $RUST_PORT is still in use after cleanup${NC}"
    echo -e "${YELLOW}Processes using port $RUST_PORT:${NC}"
    lsof -Pi :$RUST_PORT -sTCP:LISTEN || true
    exit 1
fi

if check_port $NEXTJS_PORT; then
    echo -e "${RED}✗ Port $NEXTJS_PORT is still in use after cleanup${NC}"
    echo -e "${YELLOW}Processes using port $NEXTJS_PORT:${NC}"
    lsof -Pi :$NEXTJS_PORT -sTCP:LISTEN || true
    exit 1
fi

echo -e "${GREEN}✓ No port conflicts detected${NC}"

# Create data directory
echo -e "${YELLOW}Setting up database...${NC}"
mkdir -p "$HOME/hodei-data"
export DATABASE_URL="sqlite:///$HOME/hodei-data/hodei.db"
echo -e "${GREEN}✓ Database configured: $DATABASE_URL${NC}"

# Start Rust server
echo -e "${YELLOW}Starting Rust gRPC server...${NC}"

if [ ! -f "./verified-permissions/target/release/hodei-verified-permissions" ]; then
    echo -e "${RED}✗ Rust binary not found. Run 'make build-server' first${NC}"
    exit 1
fi

start_with_retry "Rust server" \
    "env DATABASE_URL=\"$DATABASE_URL\" VERIFIED_PERMISSIONS_ADDR=\"0.0.0.0:$RUST_PORT\" RUST_LOG=info ./verified-permissions/target/release/hodei-verified-permissions > $LOG_DIR/rust-server.log 2>&1 &"

RUST_PID=$!

# Save Rust PID using PID management script
bash "$SCRIPT_DIR/manage-pids.sh" poststart "rust-server" "$RUST_PID"

echo "  PID: $RUST_PID"

# Wait for Rust server to be ready
if ! wait_for_port $RUST_PORT "Rust gRPC server"; then
    echo -e "${RED}✗ Rust server failed to start${NC}"
    echo -e "${YELLOW}Last 20 lines of log:${NC}"
    tail -20 "$LOG_DIR/rust-server.log"
    exit 1
fi

# Start Next.js frontend
echo -e "${YELLOW}Starting Next.js frontend...${NC}"
cd web-nextjs

if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}Installing dependencies...${NC}"
    npm install --legacy-peer-deps
fi

# Use npm run dev for development mode
start_with_retry "Next.js frontend" \
    "npm run dev > $LOG_DIR/nextjs-server.log 2>&1 &"

NEXTJS_PID=$!

# Save Next.js PID using PID management script
bash "$SCRIPT_DIR/manage-pids.sh" poststart "nextjs" "$NEXTJS_PID"

echo "  PID: $NEXTJS_PID"

# Wait for Next.js frontend to be ready
cd - > /dev/null
if ! wait_for_port $NEXTJS_PORT "Next.js frontend"; then
    echo -e "${RED}✗ Next.js frontend failed to start${NC}"
    echo -e "${YELLOW}Last 20 lines of log:${NC}"
    tail -20 "$LOG_DIR/nextjs-server.log"
    exit 1
fi

# ============================================================================
# Success Message
# ============================================================================

echo -e "\n${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Development Environment Ready${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "Rust gRPC Server: ${BLUE}http://localhost:$RUST_PORT${NC}"
echo -e "Next.js Frontend: ${BLUE}http://localhost:$NEXTJS_PORT${NC}"
echo -e ""
echo -e "${YELLOW}Logs:${NC}"
echo -e "  Rust:    tail -f $LOG_DIR/rust-server.log"
echo -e "  Next.js: tail -f $LOG_DIR/nextjs-server.log"
echo -e ""
echo -e "${YELLOW}Commands:${NC}"
echo -e "  make dev-logs              - View Rust server logs"
echo -e "  make dev-logs-frontend     - View Next.js logs"
echo -e "  make dev-test              - Test gRPC connection"
echo -e "  make dev-stop              - Stop all services"
echo -e ""

# ============================================================================
# Health Check Summary
# ============================================================================

echo -e "${YELLOW}Health Check Summary:${NC}"
if check_port $RUST_PORT; then
    echo -e "  ${GREEN}✓${NC} Rust gRPC server running on port $RUST_PORT"
else
    echo -e "  ${RED}✗${NC} Rust gRPC server NOT running"
fi

if check_port $NEXTJS_PORT; then
    echo -e "  ${GREEN}✓${NC} Next.js frontend running on port $NEXTJS_PORT"
else
    echo -e "  ${RED}✗${NC} Next.js frontend NOT running"
fi

echo ""

# ============================================================================
# Wait for processes
# ============================================================================

# Keep script running to monitor services
echo -e "${YELLOW}Services started successfully!${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"

# Monitor processes and exit if any dies
while true; do
    # Check if processes are still alive
    if ! kill -0 $RUST_PID 2>/dev/null; then
        echo -e "${RED}✗ Rust server process died${NC}"
        break
    fi

    if ! kill -0 $NEXTJS_PID 2>/dev/null; then
        echo -e "${RED}✗ Next.js process died${NC}"
        break
    fi

    sleep 5
done

echo -e "${RED}One or more services stopped. Exiting.${NC}"
exit 1
