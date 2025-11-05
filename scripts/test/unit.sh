#!/bin/bash
# =============================================================================
# Run Unit Tests
# This script runs all unit tests in the workspace
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/../.."

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🧪 Running unit tests...${NC}"

cd "$PROJECT_ROOT"

# Run unit tests in all packages
echo -e "${YELLOW}📦 Running verified-permissions-sdk unit tests...${NC}"
cd verified-permissions-sdk
cargo test --lib
SDK_RESULT=$?
cd "$PROJECT_ROOT"

if [ $SDK_RESULT -ne 0 ]; then
    echo -e "${RED}❌ SDK unit tests failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All unit tests passed!${NC}"
