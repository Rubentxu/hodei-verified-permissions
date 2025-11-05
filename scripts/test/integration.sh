#!/bin/bash
# =============================================================================
# Run Integration Tests
# This script runs repository and SDK integration tests
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

echo -e "${BLUE}🧪 Running integration tests...${NC}"

cd "$PROJECT_ROOT"

# Run repository tests (these don't need Docker)
echo -e "${YELLOW}📦 Running repository integration tests...${NC}"
cd verified-permissions/main

# Run tests that don't require containers
cargo test --lib

# Run simple integration tests
cargo test --test simple_integration_test

# Run repository tests
cargo test --test e2e_repository_tests

# Run policy template tests
cargo test --test policy_template_tests

# Run identity source tests
cargo test --test identity_source_integration_tests

cd "$PROJECT_ROOT"

echo -e "${GREEN}✅ All integration tests passed!${NC}"
