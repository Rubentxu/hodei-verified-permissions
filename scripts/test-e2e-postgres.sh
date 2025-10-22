#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   E2E Tests - PostgreSQL Backend                      ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"

# Cleanup function
cleanup() {
    echo -e "${YELLOW}🧹 Cleaning up...${NC}"
    docker compose -f docker-compose.postgres.yml down -v
}

# Set trap to cleanup on exit
trap cleanup EXIT

# Build services
echo -e "${YELLOW}🔨 Building services...${NC}"
docker compose -f docker-compose.postgres.yml build

# Start services
echo -e "${YELLOW}🚀 Starting PostgreSQL services...${NC}"
docker compose -f docker-compose.postgres.yml up -d

# Wait for services to be healthy
echo -e "${YELLOW}⏳ Waiting for services to be ready...${NC}"
timeout 60 bash -c 'until docker compose -f docker-compose.postgres.yml ps | grep -q "healthy"; do sleep 2; done' || {
    echo -e "${RED}❌ Services failed to start${NC}"
    docker compose -f docker-compose.postgres.yml logs
    exit 1
}

echo -e "${GREEN}✅ Services are ready${NC}"

# Show running containers
docker compose -f docker-compose.postgres.yml ps

# Run tests
echo -e "${YELLOW}🧪 Running PostgreSQL E2E tests...${NC}"
cargo test --test e2e_multi_database test_postgres -- --ignored --nocapture || {
    echo -e "${RED}❌ Tests failed${NC}"
    docker compose -f docker-compose.postgres.yml logs
    exit 1
}

echo -e "${GREEN}✅ All PostgreSQL tests passed!${NC}"
