#!/bin/bash

set -e

echo "================================"
echo "Workload Engine Testing Script"
echo "================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Docker is running
if ! docker ps > /dev/null 2>&1; then
    echo -e "${RED}✗ Docker is not running${NC}"
    echo "Please start Docker and try again"
    exit 1
fi
echo -e "${GREEN}✓ Docker is running${NC}"

# Check if test container exists
CONTAINER_NAME="workload-test"
if ! docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
    echo -e "${YELLOW}Test container not found. Creating...${NC}"
    cargo run --quiet -- create postgres --name $CONTAINER_NAME
    echo -e "${GREEN}✓ Created test container: $CONTAINER_NAME${NC}"

    # Wait for container to be ready
    echo "Waiting for database to be ready..."
    sleep 3
else
    # Check if container is running
    if ! docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
        echo -e "${YELLOW}Test container exists but is stopped. Starting...${NC}"
        cargo run --quiet -- start $CONTAINER_NAME
        sleep 2
    fi
    echo -e "${GREEN}✓ Test container is running${NC}"
fi

echo ""
echo "================================"
echo "Running Integration Tests"
echo "================================"
echo ""

# Run tests
TEST_FILTER=${1:-""}

if [ -z "$TEST_FILTER" ]; then
    echo "Running all workload integration tests..."
    cargo test --test workload_integration_test -- --ignored --nocapture
else
    echo "Running test: $TEST_FILTER"
    cargo test --test workload_integration_test $TEST_FILTER -- --ignored --nocapture
fi

TEST_RESULT=$?

echo ""
echo "================================"
echo "Test Summary"
echo "================================"

if [ $TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo "What was tested:"
    echo "  - Concurrent worker execution"
    echo "  - Rate limiting accuracy"
    echo "  - Statistics collection"
    echo "  - Workload patterns"
    echo ""
    echo "Next steps:"
    echo "  1. Implement Phase 6 (CRUD Operations)"
    echo "  2. Implement Phase 7 (CLI Integration)"
    echo "  3. Run: dbarena workload run --pattern oltp --container mydb"
else
    echo -e "${RED}✗ Some tests failed${NC}"
    echo ""
    echo "Common issues:"
    echo "  - Container not healthy (wait a bit longer)"
    echo "  - System under load (close other apps)"
    echo "  - Placeholder SQL causing errors (expected in Phase 5)"
fi

echo ""
echo "To clean up:"
echo "  cargo run -- destroy $CONTAINER_NAME -y"
echo ""

exit $TEST_RESULT
