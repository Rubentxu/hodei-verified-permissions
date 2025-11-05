#!/bin/bash
# Debug version of E2E test runner
# This script runs E2E tests with maximum verbosity

set +e  # Don't exit on errors - we want to see what's happening

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/../.."
cd "$PROJECT_ROOT"

echo "==== E2E DEBUG TEST RUNNER ===="
echo "Working directory: $(pwd)"
echo "Project root: $PROJECT_ROOT"
echo "Date: $(date)"
echo ""

# Step 1: Clean up
echo "Step 1: Cleaning up existing services..."
pkill -f "hodei-verified-permissions|nextjs" 2>/dev/null
CLEANUP_STATUS=$?
echo "Cleanup exit code: $CLEANUP_STATUS"
sleep 3
echo ""

# Step 2: Start backend
echo "Step 2: Starting backend server..."
cd verified-permissions/main
echo "Current directory: $(pwd)"
mkdir -p /home/rubentxu/hodei-data

DATABASE_URL=sqlite:///home/rubentxu/hodei-data/hodei.db nohup cargo run --bin hodei-verified-permissions > /tmp/backend-debug.log 2>&1 &
BACKEND_PID=$!
echo "Backend PID: $BACKEND_PID"
echo "Backend started at: $(date)"
echo ""

# Wait and check backend
echo "Step 3: Checking backend startup..."
for i in {1..15}; do
    echo "Check $i/15: $(date)"
    if lsof -Pi :50051 -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo "✅ Backend is listening on port 50051"
        BACKEND_READY=1
        break
    fi
    echo "Backend not ready yet, waiting..."
    sleep 2
done

if [ "${BACKEND_READY:-0}" != "1" ]; then
    echo "❌ Backend failed to start"
    echo "=== Backend log ==="
    cat /tmp/backend-debug.log
    echo "=== Process status ==="
    ps aux | grep hodei | grep -v grep
    exit 1
fi
echo ""

# Step 3: Start frontend
echo "Step 4: Starting frontend server..."
cd "$PROJECT_ROOT/web-nextjs"
echo "Current directory: $(pwd)"

nohup npm run dev > /tmp/frontend-debug.log 2>&1 &
FRONTEND_PID=$!
echo "Frontend PID: $FRONTEND_PID"
echo "Frontend started at: $(date)"
echo ""

# Wait and check frontend
echo "Step 5: Checking frontend startup..."
for i in {1..15}; do
    echo "Check $i/15: $(date)"
    if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo "✅ Frontend is listening on port 3000"
        FRONTEND_READY=1
        break
    fi
    echo "Frontend not ready yet, waiting..."
    sleep 2
done

if [ "${FRONTEND_READY:-0}" != "1" ]; then
    echo "❌ Frontend failed to start"
    echo "=== Frontend log ==="
    cat /tmp/frontend-debug.log
    echo "=== Process status ==="
    ps aux | grep next | grep -v grep
    exit 1
fi
echo ""

# Step 4: Run tests
echo "Step 6: Running E2E tests..."
cd web-nextjs
echo "Current directory: $(pwd)"

if npx playwright test --reporter=html,line 2>&1 | tee /tmp/test-results.log; then
    echo ""
    echo "✅ E2E tests completed successfully"
    TEST_EXIT_CODE=0
else
    echo ""
    echo "❌ E2E tests failed"
    TEST_EXIT_CODE=$?
fi

echo ""
echo "=== Test Results Summary ==="
tail -50 /tmp/test-results.log

echo ""
echo "=== Cleanup ==="
echo "Stopping backend (PID: $BACKEND_PID)..."
kill $BACKEND_PID 2>/dev/null || true
echo "Stopping frontend (PID: $FRONTEND_PID)..."
kill $FRONTEND_PID 2>/dev/null || true

echo ""
echo "Script completed at: $(date)"
echo "Test exit code: $TEST_EXIT_CODE"

exit $TEST_EXIT_CODE
