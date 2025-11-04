#!/bin/bash

set -e

# --- Configuration ---
RUST_PORT=${RUST_PORT:-50051}
NEXTJS_PORT=${NEXTJS_PORT:-3000}
PID_DIR="$HOME/.hodei-pids"
LOG_DIR="/tmp"
DATA_DIR="$HOME/hodei-data"

# --- Colors ---
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

# --- Utility Functions ---

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if a port is in use
port_in_use() {
    if command_exists lsof; then
        lsof -i :"$1" -sTCP:LISTEN -t >/dev/null
    elif command_exists netstat; then
        netstat -tuln | grep ":$1 " >/dev/null
    else
        echo -e "${YELLOW}Warning: Neither 'lsof' nor 'netstat' found. Cannot check for port conflicts.${NC}"
        return 1 # Assume port is not in use
    fi
}

# Function to stop all services
stop_all() {
    echo -e "${YELLOW}Stopping all services...${NC}"
    if [ -f "$PID_DIR/rust-server.pid" ]; then
        kill "$(cat "$PID_DIR/rust-server.pid")" 2>/dev/null || true
        rm -f "$PID_DIR/rust-server.pid"
    fi
    if [ -f "$PID_DIR/nextjs.pid" ]; then
        kill "$(cat "$PID_DIR/nextjs.pid")" 2>/dev/null || true
        rm -f "$PID_DIR/nextjs.pid"
    fi
    echo -e "${GREEN}✓ Services stopped.${NC}"
}

# --- Main Logic ---

# Setup trap to ensure cleanup on exit
trap stop_all INT TERM EXIT

mkdir -p "$PID_DIR"
mkdir -p "$DATA_DIR"

# 1. Start Rust Server
echo -e "${BLUE}--- Starting Rust gRPC Server ---${NC}"
if [ -f "$PID_DIR/rust-server.pid" ] && kill -0 "$(cat "$PID_DIR/rust-server.pid")" 2>/dev/null; then
    echo -e "${GREEN}✓ Rust server is already running.${NC}"
else
    if port_in_use "$RUST_PORT"; then
        echo -e "${RED}✗ Port $RUST_PORT is already in use. Aborting.${NC}"
        exit 1
    fi

    if [ ! -f "./verified-permissions/target/debug/hodei-verified-permissions" ]; then
        echo -e "${YELLOW}Rust binary not found. Building...${NC}"
        make build-server-debug
    fi

    echo -e "${YELLOW}Starting Rust server in background...${NC}"
    DATABASE_URL="sqlite://$DATA_DIR/hodei.db" ./verified-permissions/target/debug/hodei-verified-permissions > "$LOG_DIR/rust-server.log" 2>&1 &
    echo $! > "$PID_DIR/rust-server.pid"
    sleep 2

    if ! kill -0 "$(cat "$PID_DIR/rust-server.pid")" 2>/dev/null; then
        echo -e "${RED}✗ Failed to start Rust server. Check logs:${NC} $LOG_DIR/rust-server.log"
        exit 1
    fi
    echo -e "${GREEN}✓ Rust server started (PID: $(cat "$PID_DIR/rust-server.pid")).${NC}"
fi

# 2. Start Next.js Frontend
echo -e "\n${BLUE}--- Starting Next.js Frontend ---${NC}"
if [ -f "$PID_DIR/nextjs.pid" ] && kill -0 "$(cat "$PID_DIR/nextjs.pid")" 2>/dev/null; then
    echo -e "${GREEN}✓ Next.js is already running.${NC}"
else
    if port_in_use "$NEXTJS_PORT"; then
        echo -e "${RED}✗ Port $NEXTJS_PORT is already in use. Aborting.${NC}"
        exit 1
    fi

    cd web-nextjs
    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}Installing npm dependencies...${NC}"
        npm install
    fi

    echo -e "${YELLOW}Starting Next.js dev server in background...${NC}"
    npm run dev > "$LOG_DIR/nextjs-server.log" 2>&1 &
    echo $! > "$PID_DIR/nextjs.pid"
    cd ..
    sleep 5

    if ! kill -0 "$(cat "$PID_DIR/nextjs.pid")" 2>/dev/null; then
        echo -e "${RED}✗ Failed to start Next.js. Check logs:${NC} $LOG_DIR/nextjs-server.log"
        exit 1
    fi
    echo -e "${GREEN}✓ Next.js started (PID: $(cat "$PID_DIR/nextjs.pid")).${NC}"
fi

# 3. Health Checks
echo -e "\n${BLUE}--- Running Health Checks ---${NC}"
sleep 5 # Give services a moment to stabilize

echo -n "Checking Rust server... "
if curl -s http://localhost:$RUST_PORT/health | grep -q '"status":"SERVING"'; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
fi

echo -n "Checking Next.js BFF... "
if curl -s http://localhost:$NEXTJS_PORT/api/health | grep -q '"status":"SERVING"'; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
fi

# --- Done ---
echo -e "\n${GREEN}Development environment is ready!${NC}"
echo " - Rust Server Logs: tail -f $LOG_DIR/rust-server.log"
echo " - Next.js Logs:    tail -f $LOG_DIR/nextjs-server.log"
echo -e "\nPress Ctrl+C to stop all services."

# Wait indefinitely until user presses Ctrl+C
wait
