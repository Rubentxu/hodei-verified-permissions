#!/bin/bash

################################################################################
# PID Management Script for Hodei Verified Permissions
# This script manages PIDs to avoid port conflicts when starting services
################################################################################

set -e

# Configuration
PID_DIR="${HOME}/.hodei-pids"
RUST_SERVER_PIDFILE="${PID_DIR}/rust-server.pid"
NEXTJS_PIDFILE="${PID_DIR}/nextjs.pid"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

# Ensure PID directory exists
ensure_pid_dir() {
    mkdir -p "${PID_DIR}"
}

# Save PID to file
save_pid() {
    local pidfile=$1
    local pid=$2
    echo "$pid" > "$pidfile"
    echo -e "${GREEN}✓ Saved PID $pid to $pidfile${NC}"
}

# Get PID from file
get_pid() {
    local pidfile=$1
    if [ -f "$pidfile" ]; then
        cat "$pidfile"
    else
        echo ""
    fi
}

# Check if process is running
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

# Kill process if running
kill_process() {
    local pid=$1
    local name=$2
    if [ -z "$pid" ]; then
        echo -e "${YELLOW}⚠ No PID found for $name${NC}"
        return 0
    fi

    if is_process_running "$pid"; then
        echo -e "${YELLOW}Killing $name (PID: $pid)...${NC}"
        kill -TERM "$pid" 2>/dev/null || true
        sleep 2

        # Force kill if still running
        if is_process_running "$pid"; then
            echo -e "${RED}⚠ Force killing $name (PID: $pid)...${NC}"
            kill -9 "$pid" 2>/dev/null || true
            sleep 1
        fi
        echo -e "${GREEN}✓ $name killed${NC}"
    else
        echo -e "${GREEN}✓ $name is not running${NC}"
    fi
}

# Clean up PID file
cleanup_pidfile() {
    local pidfile=$1
    if [ -f "$pidfile" ]; then
        rm -f "$pidfile"
    fi
}

# List running services
list_services() {
    echo -e "${GREEN}=== Hodei Services Status ===${NC}"
    echo ""

    # Check Rust server
    RUST_PID=$(get_pid "$RUST_SERVER_PIDFILE")
    if [ -n "$RUST_PID" ]; then
        if is_process_running "$RUST_PID"; then
            echo -e "Rust Server:     ${GREEN}RUNNING${NC} (PID: $RUST_PID)"
        else
            echo -e "Rust Server:     ${RED}DEAD${NC} (stale PID: $RUST_PID)"
        fi
    else
        echo -e "Rust Server:     ${YELLOW}NOT RUNNING${NC}"
    fi

    # Check Next.js
    NEXTJS_PID=$(get_pid "$NEXTJS_PIDFILE")
    if [ -n "$NEXTJS_PID" ]; then
        if is_process_running "$NEXTJS_PID"; then
            echo -e "Next.js Frontend: ${GREEN}RUNNING${NC} (PID: $NEXTJS_PID)"
        else
            echo -e "Next.js Frontend: ${RED}DEAD${NC} (stale PID: $NEXTJS_PID)"
        fi
    else
        echo -e "Next.js Frontend: ${YELLOW}NOT RUNNING${NC}"
    fi
    echo ""
}

# Kill all Hodei services
kill_all() {
    echo -e "${GREEN}=== Stopping All Hodei Services ===${NC}"
    echo ""

    ensure_pid_dir

    # Kill Rust server
    RUST_PID=$(get_pid "$RUST_SERVER_PIDFILE")
    kill_process "$RUST_PID" "Rust Server"
    cleanup_pidfile "$RUST_SERVER_PIDFILE"

    # Kill Next.js
    NEXTJS_PID=$(get_pid "$NEXTJS_PIDFILE")
    kill_process "$NEXTJS_PID" "Next.js Frontend"
    cleanup_pidfile "$NEXTJS_PIDFILE"

    # Also kill by name pattern (safety net)
    pkill -f "hodei-verified-permissions" 2>/dev/null || true
    pkill -f "next dev" 2>/dev/null || true
    pkill -f "next" 2>/dev/null || true

    echo ""
    echo -e "${GREEN}✓ All services stopped${NC}"
}

# Pre-start cleanup (before starting new instances)
prestart_cleanup() {
    echo -e "${YELLOW}=== Pre-start Cleanup ===${NC}"
    echo ""

    ensure_pid_dir

    # Check and kill existing services
    RUST_PID=$(get_pid "$RUST_SERVER_PIDFILE")
    if [ -n "$RUST_PID" ] && is_process_running "$RUST_PID"; then
        kill_process "$RUST_PID" "Rust Server"
        cleanup_pidfile "$RUST_SERVER_PIDFILE"
    fi

    NEXTJS_PID=$(get_pid "$NEXTJS_PIDFILE")
    if [ -n "$NEXTJS_PID" ] && is_process_running "$NEXTJS_PID"; then
        kill_process "$NEXTJS_PID" "Next.js Frontend"
        cleanup_pidfile "$NEXTJS_PIDFILE"
    fi

    # Also kill by name pattern (safety net)
    pkill -f "hodei-verified-permissions" 2>/dev/null || true
    pkill -f "next dev" 2>/dev/null || true
    pkill -f "next" 2>/dev/null || true

    echo ""
    echo -e "${GREEN}✓ Cleanup completed${NC}"
}

# Post-start save (after successfully starting services)
poststart_save() {
    local service=$1
    local pid=$2

    ensure_pid_dir

    case "$service" in
        rust-server)
            save_pid "$RUST_SERVER_PIDFILE" "$pid"
            ;;
        nextjs)
            save_pid "$NEXTJS_PIDFILE" "$pid"
            ;;
    esac
}

# Main command handler
case "${1:-}" in
    prestart)
        prestart_cleanup
        ;;
    poststart)
        if [ -z "$2" ] || [ -z "$3" ]; then
            echo -e "${RED}Error: poststart requires service name and PID${NC}"
            exit 1
        fi
        poststart_save "$2" "$3"
        ;;
    kill-all)
        kill_all
        ;;
    list)
        list_services
        ;;
    status)
        list_services
        ;;
    *)
        echo "Usage: $0 {prestart|poststart|kill-all|list|status} [service] [pid]"
        echo ""
        echo "Commands:"
        echo "  prestart      - Clean up any existing processes before starting"
        echo "  poststart S P - Save PID S (service) with PID (p) after successful start"
        echo "  kill-all      - Kill all Hodei services"
        echo "  list/status   - List status of all services"
        exit 1
        ;;
esac
