#!/bin/bash

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Get build type from argument (default: release)
BUILD_TYPE="${1:-release}"

# Determine binary path based on build type
if [ "$BUILD_TYPE" = "debug" ]; then
    SERVER_BINARY="./verified-permissions/target/debug/hodei-verified-permissions"
else
    SERVER_BINARY="./verified-permissions/target/release/hodei-verified-permissions"
fi

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Running E2E Tests Against Real Server (${BUILD_TYPE^^})           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up processes and containers...${NC}"
    pkill -9 -f "hodei-verified-permissions" 2>/dev/null || true
    pkill -9 -f "next dev" 2>/dev/null || true
    
    # Stop Docker containers
    docker stop hodei-server-sqlite 2>/dev/null || true
    docker stop todo-app-sqlite 2>/dev/null || true
    docker stop hodei-verified-permissions-hodei-server-sqlite 2>/dev/null || true
    docker stop hodei-verified-permissions-todo-app-sqlite 2>/dev/null || true
    
    sleep 1
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Kill any existing processes
echo -e "${YELLOW}Killing any existing server processes...${NC}"
pkill -9 -f "hodei-verified-permissions" 2>/dev/null || true
pkill -9 -f "next dev" 2>/dev/null || true

# Stop Docker containers if they exist
echo -e "${YELLOW}Stopping Docker containers...${NC}"
docker stop hodei-server-sqlite 2>/dev/null || true
docker stop todo-app-sqlite 2>/dev/null || true
docker stop hodei-verified-permissions-hodei-server-sqlite 2>/dev/null || true
docker stop hodei-verified-permissions-todo-app-sqlite 2>/dev/null || true

# Kill any process using port 50051
echo -e "${YELLOW}Freeing port 50051...${NC}"
lsof -i :50051 2>/dev/null | grep -v COMMAND | awk '{print $2}' | xargs -r kill -9 2>/dev/null || true

# Kill any process using port 3000 (frontend)
echo -e "${YELLOW}Freeing port 3000...${NC}"
lsof -i :3000 2>/dev/null | grep -v COMMAND | awk '{print $2}' | xargs -r kill -9 2>/dev/null || true

sleep 3

# Setup database
echo -e "${YELLOW}Setting up database...${NC}"
mkdir -p /home/rubentxu/hodei-data
touch /home/rubentxu/hodei-data/hodei.db 2>/dev/null || true

# Check gRPC client dependencies
echo -e "${YELLOW}Checking gRPC client dependencies...${NC}"
cd web-nextjs
if [ ! -d "node_modules" ] || [ ! -d "node_modules/@grpc/grpc-js" ]; then
    echo -e "${YELLOW}Installing gRPC dependencies...${NC}"
    npm install @grpc/grpc-js @grpc/proto-loader > /tmp/grpc-install.log 2>&1
fi

# Verify proto file exists
if [ ! -f "../proto/authorization.proto" ]; then
    echo -e "${RED}❌ Proto file not found: ../proto/authorization.proto${NC}"
    exit 1
fi

cd ..

# Start Rust gRPC server
echo -e "${YELLOW}Starting Rust gRPC server in background (${BUILD_TYPE})...${NC}"
export DATABASE_URL="sqlite:////home/rubentxu/hodei-data/hodei.db"
$SERVER_BINARY > /tmp/hodei-server.log 2>&1 &
SERVER_PID=$!
echo $SERVER_PID > /tmp/hodei-server.pid

# Wait for Rust server to start
sleep 4

# Check if Rust server is running
if ! ps -p $SERVER_PID > /dev/null; then
    echo -e "${RED}❌ Rust server failed to start!${NC}"
    echo "Server logs:"
    cat /tmp/hodei-server.log
    exit 1
fi

echo -e "${YELLOW}Rust server started (PID: $SERVER_PID)${NC}"

# Start Next.js frontend
echo -e "${YELLOW}Building and starting Next.js frontend...${NC}"
cd web-nextjs

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}Installing dependencies...${NC}"
    npm install > /tmp/nextjs-install.log 2>&1
fi

# Build Next.js
echo -e "${YELLOW}Building Next.js...${NC}"
npm run build > /tmp/nextjs-build.log 2>&1
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Next.js build failed!${NC}"
    echo "Build logs:"
    tail -50 /tmp/nextjs-build.log
    exit 1
fi

# Start Next.js
echo -e "${YELLOW}Starting Next.js server...${NC}"
npm run start > /tmp/nextjs-server.log 2>&1 &
NEXTJS_PID=$!
echo $NEXTJS_PID > /tmp/nextjs-server.pid

# Wait for Next.js to start
sleep 8

# Check if Next.js is running
if ! ps -p $NEXTJS_PID > /dev/null; then
    echo -e "${RED}❌ Next.js server failed to start!${NC}"
    echo "Next.js server logs:"
    tail -50 /tmp/nextjs-server.log
    exit 1
fi

echo -e "${YELLOW}Next.js frontend started (PID: $NEXTJS_PID)${NC}"
cd ..

# Run E2E tests
echo -e "${YELLOW}Running E2E tests...${NC}"
npm run test:e2e 2>&1
TEST_RESULT=$?

# Report results
if [ $TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}✅ E2E tests passed!${NC}"
else
    echo -e "${RED}❌ E2E tests failed!${NC}"
fi

exit $TEST_RESULT
