#!/bin/bash
# Performance Testing Script for Policy Store
# Tests load, response times, and concurrent operations

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Performance Tests - Policy Store                          ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"

# Configuration
TEST_RESULTS_DIR="./test-results/performance"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$TEST_RESULTS_DIR/performance_report_$TIMESTAMP.json"

# Create results directory
mkdir -p "$TEST_RESULTS_DIR"

# ============================================================================
# Test 1: Concurrent Policy Store Creation
# ============================================================================
echo -e "\n${YELLOW}Test 1: Concurrent Policy Store Creation${NC}"
echo "Creating 50 policy stores concurrently..."

START_TIME=$(date +%s%N)
CONCURRENT_STORES=50

for i in $(seq 1 $CONCURRENT_STORES); do
  (
    curl -s -X POST "http://localhost:50051/v1/policy-stores" \
      -H "Content-Type: application/json" \
      -d "{\"description\": \"Load Test Store $i\"}" \
      -w "\n" >> /tmp/created_stores.log &
  ) &
done

# Wait for all background jobs
wait

END_TIME=$(date +%s%N)
DURATION=$(( (END_TIME - START_TIME) / 1000000 ))

echo -e "${GREEN}✓ Created $CONCURRENT_STORES stores in ${DURATION}ms${NC}"
echo "Average time per store: $((DURATION / CONCURRENT_STORES))ms"

# ============================================================================
# Test 2: Policy Store List Performance
# ============================================================================
echo -e "\n${YELLOW}Test 2: Policy Store List Performance${NC}"

ITERATIONS=100
TOTAL_TIME=0

for i in $(seq 1 $ITERATIONS); do
  START=$(date +%s%N)
  curl -s "http://localhost:50051/v1/policy-stores" > /dev/null
  END=$(date +%s%N)
  TIME=$(( (END - START) / 1000000 ))
  TOTAL_TIME=$((TOTAL_TIME + TIME))
done

AVERAGE_TIME=$((TOTAL_TIME / ITERATIONS))
echo -e "${GREEN}✓ Average list response time: ${AVERAGE_TIME}ms${NC}"

if [ $AVERAGE_TIME -lt 100 ]; then
  echo -e "${GREEN}  Status: EXCELLENT (< 100ms)${NC}"
elif [ $AVERAGE_TIME -lt 300 ]; then
  echo -e "${YELLOW}  Status: GOOD (< 300ms)${NC}"
else
  echo -e "${RED}  Status: POOR (> 300ms)${NC}"
fi

# ============================================================================
# Test 3: Snapshot Creation Performance
# ============================================================================
echo -e "\n${YELLOW}Test 3: Snapshot Creation Performance${NC}"

# Get first store ID
STORE_ID=$(head -1 /tmp/created_stores.log | jq -r '.policy_store_id' 2>/dev/null || echo "")

if [ -n "$STORE_ID" ] && [ "$STORE_ID" != "null" ]; then
  SNAPSHOT_ITERATIONS=20
  SNAPSHOT_START=$(date +%s%N)

  for i in $(seq 1 $SNAPSHOT_ITERATIONS); do
    curl -s -X POST "http://localhost:50051/v1/policy-stores/$STORE_ID/snapshots" \
      -H "Content-Type: application/json" \
      -d "{\"description\": \"Performance Test Snapshot $i\"}" \
      > /dev/null &
  done

  wait

  SNAPSHOT_END=$(date +%s%N)
  SNAPSHOT_DURATION=$(( (SNAPSHOT_END - SNAPSHOT_START) / 1000000 ))
  SNAPSHOT_AVERAGE=$((SNAPSHOT_DURATION / SNAPSHOT_ITERATIONS))

  echo -e "${GREEN}✓ Created $SNAPSHOT_ITERATIONS snapshots in ${SNAPSHOT_DURATION}ms${NC}"
  echo -e "${GREEN}  Average time per snapshot: ${SNAPSHOT_AVERAGE}ms${NC}"
else
  echo -e "${RED}  Could not get store ID for snapshot test${NC}"
fi

# ============================================================================
# Test 4: Memory Usage
# ============================================================================
echo -e "\n${YELLOW}Test 4: Memory Usage${NC}"

# Get process memory (in KB)
MEMORY_USAGE=$(ps -o pid,vsz,rss,comm | grep "hodei-verified-permissions" | awk '{print $3}' | head -1)

if [ -n "$MEMORY_USAGE" ]; then
  MEMORY_MB=$((MEMORY_USAGE / 1024))
  echo -e "${GREEN}✓ Current memory usage: ${MEMORY_MB}MB${NC}"

  if [ $MEMORY_MB -lt 512 ]; then
    echo -e "${GREEN}  Status: EXCELLENT (< 512MB)${NC}"
  elif [ $MEMORY_MB -lt 1024 ]; then
    echo -e "${YELLOW}  Status: GOOD (< 1GB)${NC}"
  else
    echo -e "${RED}  Status: POOR (> 1GB)${NC}"
  fi
else
  echo -e "${RED}  Could not get memory usage${NC}"
fi

# ============================================================================
# Test 5: Database Query Performance
# ============================================================================
echo -e "\n${YELLOW}Test 5: Database Query Performance${NC}"

# Test policy listing
POLICY_QUERY_START=$(date +%s%N)
curl -s "http://localhost:50051/v1/policy-stores/$STORE_ID/policies" > /dev/null
POLICY_QUERY_END=$(date +%s%N)
POLICY_QUERY_TIME=$(( (POLICY_QUERY_END - POLICY_QUERY_START) / 1000000 ))

echo -e "${GREEN}✓ Policy query time: ${POLICY_QUERY_TIME}ms${NC}"

# Test audit log query
AUDIT_QUERY_START=$(date +%s%N)
curl -s "http://localhost:50051/v1/policy-stores/$STORE_ID/audit" > /dev/null
AUDIT_QUERY_END=$(date +%s%N)
AUDIT_QUERY_TIME=$(( (AUDIT_QUERY_END - AUDIT_QUERY_START) / 1000000 ))

echo -e "${GREEN}✓ Audit log query time: ${AUDIT_QUERY_TIME}ms${NC}"

# ============================================================================
# Test 6: Stress Test - Rapid Operations
# ============================================================================
echo -e "\n${YELLOW}Test 6: Rapid Operations Stress Test${NC}"

RAPID_OPS=100
RAPID_START=$(date +%s%N)

for i in $(seq 1 $RAPID_OPS); do
  curl -s -X GET "http://localhost:50051/v1/policy-stores" > /dev/null &
done

wait

RAPID_END=$(date +%s%N)
RAPID_DURATION=$(( (RAPID_END - RAPID_START) / 1000000 ))
RAPID_AVERAGE=$((RAPID_DURATION / RAPID_OPS))

echo -e "${GREEN}✓ Completed $RAPID_OPS rapid operations in ${RAPID_DURATION}ms${NC}"
echo -e "${GREEN}  Average time per operation: ${RAPID_AVERAGE}ms${NC}"

if [ $RAPID_AVERAGE -lt 50 ]; then
  echo -e "${GREEN}  Status: EXCELLENT (< 50ms per op)${NC}"
elif [ $RAPID_AVERAGE -lt 100 ]; then
  echo -e "${YELLOW}  Status: GOOD (< 100ms per op)${NC}"
else
  echo -e "${RED}  Status: POOR (> 100ms per op)${NC}"
fi

# ============================================================================
# Generate Report
# ============================================================================
cat > "$REPORT_FILE" <<EOF
{
  "timestamp": "$TIMESTAMP",
  "test_suite": "policy-store-performance",
  "tests": {
    "concurrent_creation": {
      "stores_created": $CONCURRENT_STORES,
      "total_duration_ms": $DURATION,
      "average_ms_per_store": $((DURATION / CONCURRENT_STORES))
    },
    "list_performance": {
      "iterations": $ITERATIONS,
      "average_response_time_ms": $AVERAGE_TIME
    },
    "snapshot_creation": {
      "snapshots_created": $SNAPSHOT_ITERATIONS,
      "total_duration_ms": $SNAPSHOT_DURATION,
      "average_ms_per_snapshot": $SNAPSHOT_AVERAGE
    },
    "memory_usage": {
      "memory_mb": $MEMORY_MB
    },
    "query_performance": {
      "policy_query_ms": $POLICY_QUERY_TIME,
      "audit_query_ms": $AUDIT_QUERY_TIME
    },
    "stress_test": {
      "operations": $RAPID_OPS,
      "total_duration_ms": $RAPID_DURATION,
      "average_ms_per_op": $RAPID_AVERAGE
    }
  }
}
EOF

echo -e "\n${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Performance Test Summary                                  ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo -e "Report saved to: ${REPORT_FILE}"
echo -e "\n${GREEN}✓ Performance tests completed successfully${NC}"

# Cleanup
rm -f /tmp/created_stores.log

exit 0
