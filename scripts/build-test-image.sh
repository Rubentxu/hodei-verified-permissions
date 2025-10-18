#!/bin/bash
# Build Docker image for E2E testing

set -e

echo "🐳 Building test Docker image..."

docker build -f Dockerfile.test -t hodei-server:test .

echo "✅ Test image built successfully: hodei-server:test"
echo ""
echo "Run E2E tests with:"
echo "  cargo test --test 'e2e_*' --ignored"
