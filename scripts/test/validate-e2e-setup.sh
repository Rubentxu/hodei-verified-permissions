#!/bin/bash
# =============================================================================
# Validate E2E Test Setup
# This script validates that the E2E test environment is properly configured
# =============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/../.."
cd "$PROJECT_ROOT"

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🔍 Validating E2E Test Setup${NC}"
echo ""

ERRORS=0

# Check if Node.js is installed
echo -n "Checking Node.js... "
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    echo -e "${GREEN}✅${NC} (version $NODE_VERSION)"
else
    echo -e "${RED}❌${NC}"
    echo "  Node.js is not installed"
    ERRORS=$((ERRORS + 1))
fi

# Check if npm is installed
echo -n "Checking npm... "
if command -v npm &> /dev/null; then
    NPM_VERSION=$(npm --version)
    echo -e "${GREEN}✅${NC} (version $NPM_VERSION)"
else
    echo -e "${RED}❌${NC}"
    echo "  npm is not installed"
    ERRORS=$((ERRORS + 1))
fi

# Check if Playwright is installed
echo -n "Checking Playwright... "
if [ -d "web-nextjs/node_modules/@playwright/test" ]; then
    PLAYWRIGHT_VERSION=$(cd web-nextjs && npx playwright --version 2>/dev/null || echo "unknown")
    echo -e "${GREEN}✅${NC} (version $PLAYWRIGHT_VERSION)"
else
    echo -e "${RED}❌${NC}"
    echo "  Playwright is not installed in web-nextjs"
    ERRORS=$((ERRORS + 1))
fi

# Check if Playwright browsers are installed
echo -n "Checking Playwright browsers... "
if cd web-nextjs && npx playwright install --dry-run &>/dev/null; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${YELLOW}⚠️${NC}"
    echo "  Playwright browsers may not be installed"
    echo "  Run: make test-e2e-install"
fi
cd "$PROJECT_ROOT"

# Check if Docker is installed
echo -n "Checking Docker... "
if command -v docker &> /dev/null; then
    DOCKER_VERSION=$(docker --version)
    echo -e "${GREEN}✅${NC} ($DOCKER_VERSION)"
else
    echo -e "${YELLOW}⚠️${NC}"
    echo "  Docker is not installed (required for infrastructure)"
fi

# Check if Docker is running
echo -n "Checking if Docker is running... "
if docker ps &>/dev/null; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${YELLOW}⚠️${NC}"
    echo "  Docker is not running (required for infrastructure)"
fi

# Check if lsof is installed (for port checking)
echo -n "Checking lsof... "
if command -v lsof &> /dev/null; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${YELLOW}⚠️${NC}"
    echo "  lsof is not installed (used for port checking)"
fi

# Check for test files
echo -n "Checking E2E test files... "
if [ -d "web-nextjs/tests/e2e" ] && [ "$(ls -A web-nextjs/tests/e2e/*.spec.ts 2>/dev/null | wc -l)" -gt 0 ]; then
    TEST_COUNT=$(ls web-nextjs/tests/e2e/*.spec.ts 2>/dev/null | wc -l)
    echo -e "${GREEN}✅${NC} ($TEST_COUNT test files found)"
else
    echo -e "${RED}❌${NC}"
    echo "  No E2E test files found"
    ERRORS=$((ERRORS + 1))
fi

# Check Playwright config
echo -n "Checking Playwright config... "
if [ -f "web-nextjs/playwright.config.ts" ]; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌${NC}"
    echo "  playwright.config.ts not found"
    ERRORS=$((ERRORS + 1))
fi

# Check helper utilities
echo -n "Checking helper utilities... "
if [ -f "web-nextjs/tests/e2e/helpers.ts" ]; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${YELLOW}⚠️${NC}"
    echo "  helpers.ts not found (recommended for common test patterns)"
fi

# Check scripts
echo -n "Checking E2E script... "
if [ -f "scripts/test/e2e.sh" ] && [ -x "scripts/test/e2e.sh" ]; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${RED}❌${NC}"
    echo "  scripts/test/e2e.sh not found or not executable"
    ERRORS=$((ERRORS + 1))
fi

# Check infrastructure scripts
echo -n "Checking infrastructure scripts... "
if [ -f "scripts/test/infra-up.sh" ] && [ -f "scripts/test/infra-down.sh" ]; then
    echo -e "${GREEN}✅${NC}"
else
    echo -e "${YELLOW}⚠️${NC}"
    echo "  Infrastructure scripts not found"
fi

echo ""
echo "═══════════════════════════════════════"

if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}✅ E2E test setup is valid!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Install Playwright browsers: make test-e2e-install"
    echo "  2. Run E2E tests: make test-e2e"
    echo "  3. Run full E2E suite: make test-e2e-full"
    exit 0
else
    echo -e "${RED}❌ E2E test setup has $ERRORS error(s)${NC}"
    echo ""
    echo "Please fix the errors above before running E2E tests."
    exit 1
fi
