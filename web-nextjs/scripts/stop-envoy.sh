#!/bin/bash

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Stopping Envoy Proxy                                       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"

# Stop Envoy using PID file if it exists
if [ -f /tmp/envoy.pid ]; then
    ENVOY_PID=$(cat /tmp/envoy.pid)
    echo -e "${YELLOW}Stopping Envoy proxy (PID: $ENVOY_PID)...${NC}"
    kill $ENVOY_PID 2>/dev/null || true
    rm -f /tmp/envoy.pid
fi

# Kill any remaining Envoy processes
echo -e "${YELLOW}Killing any remaining Envoy processes...${NC}"
pkill -9 -f "envoy" 2>/dev/null || true

# Kill any process using port 8080
echo -e "${YELLOW}Freeing port 8080...${NC}"
lsof -i :8080 2>/dev/null | grep -v COMMAND | awk '{print $2}' | xargs -r kill -9 2>/dev/null || true

echo -e "${GREEN}✓ Envoy proxy stopped successfully${NC}"
