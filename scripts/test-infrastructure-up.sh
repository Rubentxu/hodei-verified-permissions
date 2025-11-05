#!/bin/bash
# =============================================================================
# Start Docker Infrastructure for Tests
# This script starts PostgreSQL and SurrealDB containers for container-based tests
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

echo -e "${BLUE}🐳 Starting Docker infrastructure for tests...${NC}"

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker is not installed or not in PATH${NC}"
    exit 1
fi

# Check if Docker daemon is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker daemon is not running${NC}"
    exit 1
fi

# Start PostgreSQL
echo -e "${YELLOW}📦 Starting PostgreSQL container...${NC}"
docker run -d \
    --name hodei-test-postgres \
    -e POSTGRES_PASSWORD=postgres \
    -e POSTGRES_DB=hodei_test \
    -p 5432:5432 \
    postgres:15-alpine

# Wait for PostgreSQL to be ready
echo -e "${YELLOW}⏳ Waiting for PostgreSQL to be ready...${NC}"
sleep 5

# Start SurrealDB
echo -e "${YELLOW}📦 Starting SurrealDB container...${NC}"
docker run -d \
    --name hodei-test-surrealdb \
    -p 8000:8000 \
    surrealdb/surrealdb:latest start

# Wait for SurrealDB to be ready
echo -e "${YELLOW}⏳ Waiting for SurrealDB to be ready...${NC}"
sleep 5

# Health checks
echo -e "${BLUE}🔍 Checking infrastructure health...${NC}"

# Check PostgreSQL
if docker exec hodei-test-postgres pg_isready -U postgres &> /dev/null; then
    echo -e "${GREEN}✅ PostgreSQL is ready (localhost:5432)${NC}"
else
    echo -e "${RED}❌ PostgreSQL failed to start${NC}"
    exit 1
fi

# Check SurrealDB
if curl -s http://localhost:8000/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ SurrealDB is ready (localhost:8000)${NC}"
else
    echo -e "${YELLOW}⚠️  SurrealDB health check failed (container might still be starting)${NC}"
fi

echo -e "${GREEN}✅ Infrastructure is ready!${NC}"
echo -e "${YELLOW}💡 Run 'make test-infrastructure-down' to stop containers${NC}"
