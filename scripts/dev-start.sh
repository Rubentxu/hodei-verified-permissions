#\!/bin/bash

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Starting Development Environment                          ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Shutting down development environment...${NC}"
    pkill -f "hodei-verified-permissions" 2>/dev/null || true
    pkill -f "next dev" 2>/dev/null || true
    sleep 1
    echo -e "${GREEN}✅ Development environment stopped${NC}"
}

# Trap to ensure cleanup on exit
trap cleanup EXIT INT TERM

# Kill any existing processes
echo -e "${YELLOW}Cleaning up existing processes...${NC}"
pkill -f "hodei-verified-permissions" 2>/dev/null || true
pkill -f "next dev" 2>/dev/null || true
sleep 2

# Create data directory
echo -e "${YELLOW}Setting up database...${NC}"
mkdir -p ./data
export DATABASE_URL="sqlite:///$(pwd)/data/hodei.db"

# Start Rust server
echo -e "${YELLOW}Starting Rust gRPC server...${NC}"
export VERIFIED_PERMISSIONS_ADDR="0.0.0.0:50051"
export RUST_LOG=info

if [ \! -f "./verified-permissions/target/release/hodei-verified-permissions" ]; then
    echo -e "${RED}✗ Rust binary not found. Run 'make build-server' first${NC}"
    exit 1
fi

./verified-permissions/target/release/hodei-verified-permissions > /tmp/rust-server.log 2>&1 &
RUST_PID=$\!
echo -e "${GREEN}✓ Rust server started (PID: $RUST_PID)${NC}"

# Wait for server to start
sleep 3

# Check if server is running
if \! kill -0 $RUST_PID 2>/dev/null; then
    echo -e "${RED}✗ Rust server failed to start${NC}"
    cat /tmp/rust-server.log
    exit 1
fi

# Start Next.js frontend
echo -e "${YELLOW}Starting Next.js frontend...${NC}"
cd web-nextjs

if [ \! -d "node_modules" ]; then
    echo -e "${YELLOW}Installing dependencies...${NC}"
    npm install --legacy-peer-deps
fi

# Use npm run dev for development mode
npm run dev > /tmp/nextjs-server.log 2>&1 &
NEXTJS_PID=$\!
echo -e "${GREEN}✓ Next.js frontend started (PID: $NEXTJS_PID)${NC}"

# Wait for frontend to start
sleep 8

echo -e "\n${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Development Environment Ready${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "Rust gRPC Server: ${BLUE}http://localhost:50051${NC}"
echo -e "Next.js Frontend: ${BLUE}http://localhost:3000${NC}"
echo -e ""
echo -e "${YELLOW}Logs:${NC}"
echo -e "  Rust:    tail -f /tmp/rust-server.log"
echo -e "  Next.js: tail -f /tmp/nextjs-server.log"
echo -e ""
echo -e "${YELLOW}Commands:${NC}"
echo -e "  make dev-logs              - View Rust server logs"
echo -e "  make dev-logs-frontend     - View Next.js logs"
echo -e "  make dev-test              - Test gRPC connection"
echo -e "  make dev-stop              - Stop all services"
echo -e ""

# Wait for processes
wait
