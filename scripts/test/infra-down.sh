#!/bin/bash
# =============================================================================
# Stop Docker Infrastructure for Tests
# Usage: ./scripts/test/infra-down.sh [profile]
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

echo -e "${BLUE}🐳 Stopping Docker infrastructure (profile: $PROFILE)...${NC}"

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

# Stop services based on profile
case $PROFILE in
    sqlite)
        echo -e "${YELLOW}⏹️  Stopping SQLite server...${NC}"
        $COMPOSE_CMD -f "$COMPOSE_DIR/docker-compose.sqlite.yml" down
        echo -e "${GREEN}✅ SQLite server stopped${NC}"
        ;;
    postgres)
        echo -e "${YELLOW}⏹️  Stopping PostgreSQL server...${NC}"
        $COMPOSE_CMD -f "$COMPOSE_DIR/docker-compose.postgres.yml" down
        echo -e "${GREEN}✅ PostgreSQL server stopped${NC}"
        ;;
    surrealdb)
        echo -e "${YELLOW}⏹️  Stopping SurrealDB server...${NC}"
        $COMPOSE_CMD -f "$COMPOSE_DIR/docker-compose.surrealdb.yml" down
        echo -e "${GREEN}✅ SurrealDB server stopped${NC}"
        ;;
    all)
        echo -e "${YELLOW}⏹️  Stopping all servers...${NC}"
        $COMPOSE_CMD -f "$COMPOSE_DIR/docker-compose.sqlite.yml" -f "$COMPOSE_DIR/docker-compose.postgres.yml" -f "$COMPOSE_DIR/docker-compose.surrealdb.yml" down
        echo -e "${GREEN}✅ All servers stopped${NC}"
        ;;
    *)
        echo -e "${RED}❌ Unknown profile: $PROFILE${NC}"
        echo "Valid profiles: sqlite, postgres, surrealdb, all"
        exit 1
        ;;
esac

echo -e "${GREEN}✅ Infrastructure stopped!${NC}"
