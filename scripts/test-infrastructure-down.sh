#!/bin/bash
# =============================================================================
# Stop Docker Infrastructure for Tests
# This script stops all Docker containers started by test-infrastructure-up.sh
# =============================================================================

set -e

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🐳 Stopping Docker infrastructure...${NC}"

# Stop PostgreSQL container
if docker ps -q -f name=hodei-test-postgres | grep -q .; then
    echo -e "${YELLOW}⏹️  Stopping PostgreSQL container...${NC}"
    docker stop hodei-test-postgres > /dev/null 2>&1 || true
    docker rm hodei-test-postgres > /dev/null 2>&1 || true
    echo -e "${GREEN}✅ PostgreSQL stopped${NC}"
else
    echo -e "${YELLOW}ℹ️  PostgreSQL container not running${NC}"
fi

# Stop SurrealDB container
if docker ps -q -f name=hodei-test-surrealdb | grep -q .; then
    echo -e "${YELLOW}⏹️  Stopping SurrealDB container...${NC}"
    docker stop hodei-test-surrealdb > /dev/null 2>&1 || true
    docker rm hodei-test-surrealdb > /dev/null 2>&1 || true
    echo -e "${GREEN}✅ SurrealDB stopped${NC}"
else
    echo -e "${YELLOW}ℹ️  SurrealDB container not running${NC}"
fi

echo -e "${GREEN}✅ Infrastructure stopped!${NC}"
