#!/bin/bash
# =============================================================================
# Run Container-Based Tests
# This script runs tests that require Docker (PostgreSQL, SurrealDB)
# Usage: ./scripts/test/container.sh [profile]
# Profiles: postgres | surrealdb | all (default: all)
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/../.."
PROFILE=${1:-all}

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🧪 Running container-based tests (profile: $PROFILE)...${NC}"

cd "$PROJECT_ROOT"

# Start infrastructure
echo -e "${YELLOW}🚀 Starting infrastructure...${NC}"
"$SCRIPT_DIR/infra-up.sh" "$PROFILE"

# Wait for infrastructure to be ready
echo -e "${YELLOW}⏳ Waiting for infrastructure...${NC}"
sleep 10

# Run container-based tests
echo -e "${YELLOW}📦 Running container-based tests...${NC}"
cd verified-permissions/main

# Build with containers feature
cargo test --features containers --test '*'

cd "$PROJECT_ROOT"

# Stop infrastructure
echo -e "${YELLOW}🧹 Cleaning up infrastructure...${NC}"
"$SCRIPT_DIR/infra-down.sh" "$PROFILE"

echo -e "${GREEN}✅ All container-based tests passed!${NC}"
