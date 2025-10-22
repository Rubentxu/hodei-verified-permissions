#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   E2E Tests - All Database Backends                   ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"

FAILED_TESTS=()
PASSED_TESTS=()

# Test SQLite
echo -e "\n${YELLOW}═══════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}Testing SQLite Backend${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════${NC}"
if ./scripts/test-e2e-sqlite.sh; then
    PASSED_TESTS+=("SQLite")
    echo -e "${GREEN}✅ SQLite tests passed${NC}"
else
    FAILED_TESTS+=("SQLite")
    echo -e "${RED}❌ SQLite tests failed${NC}"
fi

# Test PostgreSQL
echo -e "\n${YELLOW}═══════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}Testing PostgreSQL Backend${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════${NC}"
if ./scripts/test-e2e-postgres.sh; then
    PASSED_TESTS+=("PostgreSQL")
    echo -e "${GREEN}✅ PostgreSQL tests passed${NC}"
else
    FAILED_TESTS+=("PostgreSQL")
    echo -e "${RED}❌ PostgreSQL tests failed${NC}"
fi

# Test SurrealDB
echo -e "\n${YELLOW}═══════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}Testing SurrealDB Backend${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════${NC}"
if ./scripts/test-e2e-surrealdb.sh; then
    PASSED_TESTS+=("SurrealDB")
    echo -e "${GREEN}✅ SurrealDB tests passed${NC}"
else
    FAILED_TESTS+=("SurrealDB")
    echo -e "${RED}❌ SurrealDB tests failed${NC}"
fi

# Summary
echo -e "\n${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Test Summary                                         ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"

if [ ${#PASSED_TESTS[@]} -gt 0 ]; then
    echo -e "${GREEN}✅ Passed (${#PASSED_TESTS[@]}):${NC}"
    for test in "${PASSED_TESTS[@]}"; do
        echo -e "   - $test"
    done
fi

if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    echo -e "${RED}❌ Failed (${#FAILED_TESTS[@]}):${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo -e "   - $test"
    done
    exit 1
else
    echo -e "\n${GREEN}🎉 All database backends passed!${NC}"
    exit 0
fi
