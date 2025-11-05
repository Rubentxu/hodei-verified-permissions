#!/bin/bash
# =============================================================================
# Stop Services for E2E Tests
# This script stops all services started by test-services-up.sh
# =============================================================================

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

PID_DIR="${HOME}/.hodei-pids"

echo -e "${BLUE}⏹️  Stopping services...${NC}"

# Stop backend
if [ -f "$PID_DIR/hodei-backend.pid" ]; then
    BACKEND_PID=$(cat "$PID_DIR/hodei-backend.pid")
    if ps -p "$BACKEND_PID" > /dev/null 2>&1; then
        echo -e "${YELLOW}⏹️  Stopping backend (PID: $BACKEND_PID)...${NC}"
        kill -TERM "$BACKEND_PID" 2>/dev/null || true
        sleep 2
        if ps -p "$BACKEND_PID" > /dev/null 2>&1; then
            kill -9 "$BACKEND_PID" 2>/dev/null || true
        fi
        echo -e "${GREEN}✅ Backend stopped${NC}"
    fi
    rm -f "$PID_DIR/hodei-backend.pid"
else
    echo -e "${YELLOW}ℹ️  Backend PID file not found${NC}"
fi

# Stop frontend
if [ -f "$PID_DIR/hodei-frontend.pid" ]; then
    FRONTEND_PID=$(cat "$PID_DIR/hodei-frontend.pid")
    if ps -p "$FRONTEND_PID" > /dev/null 2>&1; then
        echo -e "${YELLOW}⏹️  Stopping frontend (PID: $FRONTEND_PID)...${NC}"
        kill -TERM "$FRONTEND_PID" 2>/dev/null || true
        sleep 2
        if ps -p "$FRONTEND_PID" > /dev/null 2>&1; then
            kill -9 "$FRONTEND_PID" 2>/dev/null || true
        fi
        echo -e "${GREEN}✅ Frontend stopped${NC}"
    fi
    rm -f "$PID_DIR/hodei-frontend.pid"
else
    echo -e "${YELLOW}ℹ️  Frontend PID file not found${NC}"
fi

echo -e "${GREEN}✅ All services stopped!${NC}"
