#!/bin/bash
# =============================================================================
# Start Docker Infrastructure for Tests
# Usage: ./scripts/test/infra-up.sh [profile]
# Profiles: sqlite | postgres | surrealdb | all (default: sqlite)
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/../.."
COMPOSE_DIR="$SCRIPT_DIR/docker-compose"
PROFILE=${1:-sqlite}

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🐳 Starting Docker infrastructure for tests (profile: $PROFILE)...${NC}"

# Check Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker is not installed${NC}"
    exit 1
fi

if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}❌ Docker daemon is not running${NC}"
    exit 1
fi

# Check docker-compose
if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo -e "${RED}❌ Docker Compose is not installed${NC}"
    exit 1
fi

# Determine compose command
COMPOSE_CMD="docker-compose"
if docker compose version &> /dev/null; then
    COMPOSE_CMD="docker compose"
fi

# Change to project root for proper path resolution
cd "$PROJECT_ROOT"

# Start services based on profile
case $PROFILE in
    sqlite)
        echo -e "${YELLOW}📦 Starting SQLite server...${NC}"
        $COMPOSE_CMD -f "$COMPOSE_DIR/docker-compose.sqlite.yml" up -d
        echo -e "${GREEN}✅ SQLite server started${NC}"
        ;;
    postgres)
        echo -e "${YELLOW}📦 Starting PostgreSQL server...${NC}"
        $COMPOSE_CMD -f "$COMPOSE_DIR/docker-compose.postgres.yml" up -d
        echo -e "${GREEN}✅ PostgreSQL server started${NC}"
        ;;
    surrealdb)
        echo -e "${YELLOW}📦 Starting SurrealDB server...${NC}"
        $COMPOSE_CMD -f "$COMPOSE_DIR/docker-compose.surrealdb.yml" up -d
        echo -e "${GREEN}✅ SurrealDB server started${NC}"
        ;;
    all)
        echo -e "${YELLOW}📦 Starting all servers...${NC}"
        $COMPOSE_CMD -f "$COMPOSE_DIR/docker-compose.sqlite.yml" -f "$COMPOSE_DIR/docker-compose.postgres.yml" -f "$COMPOSE_DIR/docker-compose.surrealdb.yml" up -d
        echo -e "${GREEN}✅ All servers started${NC}"
        echo -e "${YELLOW}📍 SQLite:  localhost:50051${NC}"
        echo -e "${YELLOW}📍 Postgres: localhost:50052${NC}"
        echo -e "${YELLOW}📍 SurrealDB: localhost:8001${NC}"
        ;;
    *)
        echo -e "${RED}❌ Unknown profile: $PROFILE${NC}"
        echo "Valid profiles: sqlite, postgres, surrealdb, all"
        exit 1
        ;;
esac

# Wait for services to be healthy
echo -e "${YELLOW}⏳ Waiting for services to be ready...${NC}"
sleep 5

# Health checks based on profile
case $PROFILE in
    sqlite|all)
        if curl -s http://localhost:50051/health > /dev/null 2>&1; then
            echo -e "${GREEN}✅ SQLite server is ready${NC}"
        else
            echo -e "${YELLOW}⚠️  SQLite server might still be starting${NC}"
        fi
        ;;
esac

case $PROFILE in
    postgres|all)
        if curl -s http://localhost:50052/health > /dev/null 2>&1; then
            echo -e "${GREEN}✅ PostgreSQL server is ready${NC}"
        else
            echo -e "${YELLOW}⚠️  PostgreSQL server might still be starting${NC}"
        fi
        ;;
esac

case $PROFILE in
    surrealdb|all)
        if curl -s http://localhost:8001/health > /dev/null 2>&1; then
            echo -e "${GREEN}✅ SurrealDB server is ready${NC}"
        else
            echo -e "${YELLOW}⚠️  SurrealDB server might still be starting${NC}"
        fi
        ;;
esac

echo -e "${GREEN}✅ Infrastructure ready!${NC}"
echo -e "${YELLOW}💡 Run './scripts/test/infra-down.sh $PROFILE' to stop${NC}"
