#!/bin/bash
set -e

echo "🚀 Starting E2E Tests for Hodei Verified Permissions"
echo "=================================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker is not running. Please start Docker first.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Docker is running${NC}"

# Clean up any existing containers
echo -e "${YELLOW}🧹 Cleaning up existing containers...${NC}"
docker-compose -f docker-compose.test.yml down -v 2>/dev/null || true

# Build images
echo -e "${YELLOW}🔨 Building Docker images...${NC}"
docker-compose -f docker-compose.test.yml build

# Start services
echo -e "${YELLOW}🚀 Starting services...${NC}"
docker-compose -f docker-compose.test.yml up -d hodei-server todo-app

# Wait for services to be healthy
echo -e "${YELLOW}⏳ Waiting for services to be ready...${NC}"
timeout 60 bash -c 'until docker-compose -f docker-compose.test.yml ps | grep -q "healthy"; do sleep 2; done' || {
    echo -e "${RED}❌ Services failed to start${NC}"
    docker-compose -f docker-compose.test.yml logs
    docker-compose -f docker-compose.test.yml down -v
    exit 1
}

echo -e "${GREEN}✅ Services are ready${NC}"

# Show service status
echo -e "${YELLOW}📊 Service Status:${NC}"
docker-compose -f docker-compose.test.yml ps

# Run E2E tests
echo -e "${YELLOW}🧪 Running E2E tests...${NC}"
cargo test --test e2e_full_stack -- --ignored --nocapture || {
    echo -e "${RED}❌ E2E tests failed${NC}"
    echo -e "${YELLOW}📋 Server logs:${NC}"
    docker-compose -f docker-compose.test.yml logs hodei-server
    echo -e "${YELLOW}📋 TODO app logs:${NC}"
    docker-compose -f docker-compose.test.yml logs todo-app
    docker-compose -f docker-compose.test.yml down -v
    exit 1
}

echo -e "${GREEN}✅ All E2E tests passed!${NC}"

# Cleanup
echo -e "${YELLOW}🧹 Cleaning up...${NC}"
docker-compose -f docker-compose.test.yml down -v

echo -e "${GREEN}🎉 E2E Test Suite Completed Successfully!${NC}"
