#!/bin/bash

set -e

# ============================================================================
# Configuration
# ============================================================================
RUST_PORT=${RUST_PORT:-50051}
NEXTJS_PORT=${NEXTJS_PORT:-3000}
LOG_DIR=${LOG_DIR:-/tmp}
DATA_DIR=${DATA_DIR:-./data}
PID_DIR="${HOME}/.hodei-pids"
RUST_SERVER_PIDFILE="${PID_DIR}/rust-server.pid"
NEXTJS_PIDFILE="${PID_DIR}/nextjs.pid"

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# ============================================================================
# Utility Functions
# ============================================================================

ensure_pid_dir() {
    mkdir -p "${PID_DIR}"
}

save_pid() {
    local pidfile=$1
    local pid=$2
    echo "$pid" > "$pidfile"
}

get_pid() {
    local pidfile=$1
    if [ -f "$pidfile" ]; then
        cat "$pidfile"
    else
        echo ""
    fi
}

is_process_running() {
    local pid=$1
    if [ -z "$pid" ]; then
        return 1
    fi
    if ps -p "$pid" > /dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

kill_process() {
    local pid=$1
    local name=$2
    if [ -z "$pid" ]; then
        return 0
    fi

    if is_process_running "$pid"; then
        echo -e "${YELLOW}Killing $name (PID: $pid)...${NC}"
        kill -TERM "$pid" 2>/dev/null || true
        sleep 2

        if is_process_running "$pid"; then
            echo -e "${RED}Force killing $name (PID: $pid)...${NC}"
            kill -9 "$pid" 2>/dev/null || true
            sleep 1
        fi
    fi
}

cleanup_pidfile() {
    local pidfile=$1
    if [ -f "$pidfile" ]; then
        rm -f "$pidfile"
    fi
}

# ============================================================================
# Service Functions
# ============================================================================

start_backend() {
    ensure_pid_dir
    local pid=$(get_pid "$RUST_SERVER_PIDFILE")
    if is_process_running "$pid"; then
        echo -e "${YELLOW}Backend already running. Restarting...${NC}"
        kill_process "$pid" "Backend"
        cleanup_pidfile "$RUST_SERVER_PIDFILE"
    fi

    echo -e "${BLUE}--- Starting Backend ---${NC}"
    if [ ! -f "./verified-permissions/target/debug/hodei-verified-permissions" ]; then
        echo -e "${YELLOW}Backend binary not found. Building...${NC}"
        make build-server-debug
    fi

    DATABASE_URL="sqlite://${DATA_DIR}/hodei.db" ./verified-permissions/target/debug/hodei-verified-permissions > "${LOG_DIR}/rust-server.log" 2>&1 &
    local new_pid=$!
    save_pid "$RUST_SERVER_PIDFILE" "$new_pid"
    echo -e "${GREEN}✓ Backend started (PID: $new_pid).${NC}"
}

start_frontend() {
    ensure_pid_dir
    local pid=$(get_pid "$NEXTJS_PIDFILE")
    if is_process_running "$pid"; then
        echo -e "${GREEN}✓ Frontend is already running (PID: $pid).${NC}"
        return
    fi

    echo -e "${BLUE}--- Starting Frontend ---${NC}"
    cd web-nextjs
    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}Installing npm dependencies...${NC}"
        npm install
    fi

    npm run dev > "${LOG_DIR}/nextjs-server.log" 2>&1 &
    local new_pid=$!
    cd ..
    save_pid "$NEXTJS_PIDFILE" "$new_pid"
    echo -e "${GREEN}✓ Frontend started (PID: $new_pid).${NC}"
}

stop_backend() {
    ensure_pid_dir
    local pid=$(get_pid "$RUST_SERVER_PIDFILE")
    if [ -z "$pid" ]; then
        echo -e "${YELLOW}Backend is not running.${NC}"
        return
    fi
    kill_process "$pid" "Backend"
    cleanup_pidfile "$RUST_SERVER_PIDFILE"
    echo -e "${GREEN}✓ Backend stopped.${NC}"
}

stop_frontend() {
    ensure_pid_dir
    local pid=$(get_pid "$NEXTJS_PIDFILE")
    if [ -z "$pid" ]; then
        echo -e "${YELLOW}Frontend is not running.${NC}"
        return
    fi
    kill_process "$pid" "Frontend"
    cleanup_pidfile "$NEXTJS_PIDFILE"
    echo -e "${GREEN}✓ Frontend stopped.${NC}"
}

stop_all() {
    stop_backend
    stop_frontend
}

show_status() {
    ensure_pid_dir
    echo -e "${BLUE}--- Services Status ---${NC}"
    local backend_pid=$(get_pid "$RUST_SERVER_PIDFILE")
    if is_process_running "$backend_pid"; then
        echo -e "Backend:  ${GREEN}RUNNING${NC} (PID: $backend_pid, Port: $RUST_PORT)"
    else
        echo -e "Backend:  ${RED}STOPPED${NC}"
    fi

    local frontend_pid=$(get_pid "$NEXTJS_PIDFILE")
    if is_process_running "$frontend_pid"; then
        echo -e "Frontend: ${GREEN}RUNNING${NC} (PID: $frontend_pid, Port: $NEXTJS_PORT)"
    else
        echo -e "Frontend: ${RED}STOPPED${NC}"
    fi
}

health_check() {
    echo -e "${BLUE}--- Health Checks ---${NC}"
    sleep 5 # Give services a moment to stabilize

    echo -n "Checking Backend... "
    if nc -z localhost $RUST_PORT > /dev/null 2>&1; then
        echo -e "${GREEN}✓ OK${NC}"
    else
        echo -e "${RED}✗ FAILED${NC}"
    fi

    echo -n "Checking Frontend... "
    if curl -s http://localhost:$NEXTJS_PORT/api/health | grep -q '"status"'; then
        echo -e "${GREEN}✓ OK${NC}"
    else
        echo -e "${RED}✗ FAILED${NC}"
    fi
}

start_all() {
    start_backend
    start_frontend
    echo -e "\n${BLUE}--- Endpoints ---${NC}"
    echo -e "Backend:  http://localhost:${RUST_PORT}"
    echo -e "Frontend: http://localhost:${NEXTJS_PORT}"
    echo ""
    health_check
}

# ============================================================================
# Main Logic
# ============================================================================

case "$1" in
    start-all)
        start_all
        ;;
    start-backend)
        start_backend
        ;;
    start-frontend)
        start_frontend
        ;;
    stop-backend)
        stop_backend
        ;;
    stop-frontend)
        stop_frontend
        ;;
    stop-all)
        stop_all
        ;;
    status)
        show_status
        ;;
    *)
        echo "Usage: $0 {start-all|start-backend|start-frontend|stop-backend|stop-frontend|stop-all|status}"
        exit 1
        ;;
esac
