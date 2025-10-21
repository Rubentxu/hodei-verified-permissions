#!/bin/bash
set -e

echo "🔑 Starting Identity Providers E2E Tests"
echo "=================================================="

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker is not running. Please start Docker first.${NC}"
    echo ""
    echo "To start Docker:"
    echo "  Linux:   sudo systemctl start docker"
    echo "  macOS:   open -a Docker"
    echo "  Windows: Start Docker Desktop"
    exit 1
fi

echo -e "${GREEN}✅ Docker is running${NC}"

# Clean up any existing containers
echo -e "${YELLOW}🧹 Cleaning up existing containers...${NC}"
docker compose -f docker-compose.identity-providers.yml down -v 2>/dev/null || true

# Build images
echo -e "${YELLOW}🔨 Building Docker images...${NC}"
echo "This may take 5-10 minutes on first run..."
docker compose -f docker-compose.identity-providers.yml build

# Start services
echo -e "${YELLOW}🚀 Starting Identity Provider services...${NC}"
echo "Starting: Keycloak, Zitadel, and Hodei servers..."
docker compose -f docker-compose.identity-providers.yml up -d

# Wait for services to be healthy
echo -e "${YELLOW}⏳ Waiting for services to be ready...${NC}"
echo "This may take 2-3 minutes for Keycloak and Zitadel to initialize..."

# Function to check service health
check_service() {
    local service=$1
    local max_attempts=60
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if docker compose -f docker-compose.identity-providers.yml ps | grep "$service" | grep -q "healthy\|Up"; then
            echo -e "${GREEN}✅ $service is ready${NC}"
            return 0
        fi
        attempt=$((attempt + 1))
        echo -n "."
        sleep 2
    done
    
    echo -e "${RED}❌ $service failed to start${NC}"
    return 1
}

# Check each service
echo ""
echo "Checking services..."
check_service "keycloak" || {
    echo -e "${RED}❌ Keycloak failed to start${NC}"
    docker compose -f docker-compose.identity-providers.yml logs keycloak
    docker compose -f docker-compose.identity-providers.yml down -v
    exit 1
}

check_service "zitadel" || {
    echo -e "${RED}❌ Zitadel failed to start${NC}"
    docker compose -f docker-compose.identity-providers.yml logs zitadel
    docker compose -f docker-compose.identity-providers.yml down -v
    exit 1
}

check_service "hodei-server-keycloak" || {
    echo -e "${RED}❌ Hodei server (Keycloak) failed to start${NC}"
    docker compose -f docker-compose.identity-providers.yml logs hodei-server-keycloak
    docker compose -f docker-compose.identity-providers.yml down -v
    exit 1
}

check_service "hodei-server-zitadel" || {
    echo -e "${RED}❌ Hodei server (Zitadel) failed to start${NC}"
    docker compose -f docker-compose.identity-providers.yml logs hodei-server-zitadel
    docker compose -f docker-compose.identity-providers.yml down -v
    exit 1
}

echo ""
echo -e "${GREEN}✅ All services are ready${NC}"

# Show service status
echo -e "${YELLOW}📊 Service Status:${NC}"
docker compose -f docker-compose.identity-providers.yml ps

echo ""
echo -e "${BLUE}🌐 Service URLs:${NC}"
echo "  Keycloak:              http://localhost:8080 (admin/admin)"
echo "  Zitadel:               http://localhost:8082"
echo "  Hodei (Keycloak):      http://localhost:50054"
echo "  Hodei (Zitadel):       http://localhost:50055"
echo "  TODO App (Keycloak):   http://localhost:3003"
echo "  TODO App (Zitadel):    http://localhost:3004"

# Run E2E tests
echo ""
echo -e "${YELLOW}🧪 Running Identity Provider E2E tests...${NC}"
echo ""

# Run all tests
echo -e "${BLUE}Running all Identity Provider tests...${NC}"
cargo test --test e2e_identity_providers -- --ignored --nocapture || {
    echo -e "${RED}❌ Identity Provider tests failed${NC}"
    echo ""
    echo -e "${YELLOW}📋 Keycloak logs:${NC}"
    docker compose -f docker-compose.identity-providers.yml logs --tail=50 keycloak
    echo ""
    echo -e "${YELLOW}📋 Zitadel logs:${NC}"
    docker compose -f docker-compose.identity-providers.yml logs --tail=50 zitadel
    echo ""
    echo -e "${YELLOW}📋 Hodei server (Keycloak) logs:${NC}"
    docker compose -f docker-compose.identity-providers.yml logs --tail=50 hodei-server-keycloak
    echo ""
    echo -e "${YELLOW}📋 Hodei server (Zitadel) logs:${NC}"
    docker compose -f docker-compose.identity-providers.yml logs --tail=50 hodei-server-zitadel
    
    echo ""
    echo -e "${YELLOW}Keep services running? (y/n)${NC}"
    read -r keep_running
    if [[ ! $keep_running =~ ^[Yy]$ ]]; then
        docker compose -f docker-compose.identity-providers.yml down -v
    fi
    exit 1
}

echo ""
echo -e "${GREEN}✅ All Identity Provider tests passed!${NC}"
echo -e "${GREEN}  ✅ Keycloak integration tests${NC}"
echo -e "${GREEN}  ✅ Zitadel integration tests${NC}"
echo -e "${GREEN}  ✅ Claims mapping tests${NC}"
echo -e "${GREEN}  ✅ Auto-detection tests${NC}"

# Ask if user wants to keep services running
echo ""
echo -e "${YELLOW}Keep services running for manual testing? (y/n)${NC}"
read -r keep_running

if [[ $keep_running =~ ^[Yy]$ ]]; then
    echo ""
    echo -e "${GREEN}Services are still running!${NC}"
    echo ""
    echo "You can:"
    echo "  - Access Keycloak at http://localhost:8080 (admin/admin)"
    echo "  - Access Zitadel at http://localhost:8082"
    echo "  - Test TODO app with Keycloak at http://localhost:3003"
    echo "  - Test TODO app with Zitadel at http://localhost:3004"
    echo ""
    echo "To stop services later, run:"
    echo "  docker compose -f docker-compose.identity-providers.yml down -v"
else
    # Cleanup
    echo ""
    echo -e "${YELLOW}🧹 Cleaning up...${NC}"
    docker compose -f docker-compose.identity-providers.yml down -v
    echo -e "${GREEN}✅ Cleanup complete${NC}"
fi

echo ""
echo -e "${GREEN}🎉 Identity Provider E2E Test Suite Completed Successfully!${NC}"
