#!/bin/bash

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Starting Envoy Proxy for gRPC-Web to gRPC translation      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"

# Check if Envoy is installed
if ! command -v envoy &> /dev/null; then
    echo -e "${RED}Error: Envoy is not installed. Please install Envoy first.${NC}"
    echo -e "${YELLOW}Visit: https://www.envoyproxy.io/docs/envoy/latest/start/install${NC}"
    exit 1
fi

# Kill any existing Envoy processes
echo -e "${YELLOW}Killing any existing Envoy processes...${NC}"
pkill -9 -f "envoy" 2>/dev/null || true

# Kill any process using port 8080
echo -e "${YELLOW}Freeing port 8080...${NC}"
lsof -i :8080 2>/dev/null | grep -v COMMAND | awk '{print $2}' | xargs -r kill -9 2>/dev/null || true

# Start Envoy proxy
echo -e "${YELLOW}Starting Envoy proxy on port 8080...${NC}"
envoy -c envoy.yaml > /tmp/envoy.log 2>&1 &
ENVOY_PID=$!
echo $ENVOY_PID > /tmp/envoy.pid

echo -e "${GREEN}Envoy proxy started with PID: $ENVOY_PID${NC}"
echo -e "${GREEN}gRPC-Web endpoint: http://localhost:8080${NC}"
echo -e "${GREEN}Admin interface: http://localhost:9901${NC}"

# Wait a moment for Envoy to start
sleep 2

# Check if Envoy is running
if ps -p $ENVOY_PID > /dev/null; then
    echo -e "${GREEN}✓ Envoy proxy is running successfully${NC}"
else
    echo -e "${RED}✗ Failed to start Envoy proxy${NC}"
    echo -e "${RED}Check logs: tail -f /tmp/envoy.log${NC}"
    exit 1
fi
