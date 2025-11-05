#!/bin/bash
# =============================================================================
# Start Services for E2E Tests
# This script starts the backend and frontend services
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/.."

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

# Configuration
RUST_PORT=${RUST_PORT:-50051}
NEXTJS_PORT=${NEXTJS_PORT:-3000}
PID_DIR="${HOME}/.hodei-pids"
DATA_DIR="${PROJECT_ROOT}/data"

echo -e "${BLUE}🚀 Starting services for E2E tests...${NC}"

# Ensure directories exist
mkdir -p "$DATA_DIR"
mkdir -p "$PID_DIR"

# Start backend
echo -e "${YELLOW}📦 Starting backend server...${NC}"

# Check if already running
if [ -f "$PID_DIR/hodei-backend.pid" ] && ps -p "$(cat "$PID_DIR/hodei-backend.pid")" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Backend already running${NC}"
else
    # Build if needed
    if [ ! -f "$PROJECT_ROOT/verified-permissions/target/debug/hodei-verified-permissions" ]; then
        echo -e "${YELLOW}🔨 Building backend...${NC}"
        cd "$PROJECT_ROOT/verified-permissions"
        cargo build
        cd "$PROJECT_ROOT"
    fi

    # Start backend
    DATABASE_URL="sqlite://$DATA_DIR/test.db" \
    RUST_LOG=info \
    "$PROJECT_ROOT/verified-permissions/target/debug/hodei-verified-permissions" > /tmp/hodei-backend.log 2>&1 &

    BACKEND_PID=$!
    echo $BACKEND_PID > "$PID_DIR/hodei-backend.pid"

    # Wait for backend to be ready
    echo -e "${YELLOW}⏳ Waiting for backend to be ready...${NC}"
    for i in {1..30}; do
        if curl -s http://localhost:$RUST_PORT/health > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Backend is ready${NC}"
            break
        fi
        if [ $i -eq 30 ]; then
            echo -e "${RED}❌ Backend failed to start${NC}"
            echo -e "${YELLOW}📝 Check logs: /tmp/hodei-backend.log${NC}"
            exit 1
        fi
        sleep 1
    done
fi

# Start frontend
echo -e "${YELLOW}📦 Starting frontend server...${NC}"

if [ -f "$PID_DIR/hodei-frontend.pid" ] && ps -p "$(cat "$PID_DIR/hodei-frontend.pid")" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Frontend already running${NC}"
else
    cd "$PROJECT_ROOT/web-nextjs"

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}📦 Installing frontend dependencies...${NC}"
        npm install
    fi

    # Start frontend
    npm run dev > /tmp/hodei-frontend.log 2>&1 &

    FRONTEND_PID=$!
    echo $FRONTEND_PID > "$PID_DIR/hodei-frontend.pid"
    cd "$PROJECT_ROOT"

    # Wait for frontend to be ready
    echo -e "${YELLOW}⏳ Waiting for frontend to be ready...${NC}"
    for i in {1..30}; do
        if curl -s http://localhost:$NEXTJS_PORT/api/health > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Frontend is ready${NC}"
            break
        fi
        if [ $i -eq 30 ]; then
            echo -e "${RED}❌ Frontend failed to start${NC}"
            echo -e "${YELLOW}📝 Check logs: /tmp/hodei-frontend.log${NC}"
            exit 1
        fi
        sleep 1
    done
fi

echo -e "${GREEN}✅ All services are ready!${NC}"
echo -e "${YELLOW}📍 Backend:  http://localhost:$RUST_PORT${NC}"
echo -e "${YELLOW}📍 Frontend: http://localhost:$NEXTJS_PORT${NC}"
echo -e "${YELLOW}💡 Run 'make test-services-down' to stop services${NC}"
