#!/bin/bash
# Manual Test Script for dbarena v0.2.0
# This script walks through critical manual test cases

set -e

DBARENA="./target/release/dbarena"
TEST_LOG="manual_test_results.log"
PASSED=0
FAILED=0

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "================================================================" | tee $TEST_LOG
echo "dbarena v0.2.0 Manual Test Suite" | tee -a $TEST_LOG
echo "Date: $(date)" | tee -a $TEST_LOG
echo "================================================================" | tee -a $TEST_LOG
echo "" | tee -a $TEST_LOG

# Helper functions
pass_test() {
    echo -e "${GREEN}✓ PASS:${NC} $1" | tee -a $TEST_LOG
    ((PASSED++))
}

fail_test() {
    echo -e "${RED}✗ FAIL:${NC} $1" | tee -a $TEST_LOG
    echo "  Error: $2" | tee -a $TEST_LOG
    ((FAILED++))
}

info() {
    echo -e "${YELLOW}ℹ INFO:${NC} $1" | tee -a $TEST_LOG
}

run_test() {
    echo "" | tee -a $TEST_LOG
    echo "----------------------------------------" | tee -a $TEST_LOG
    echo "TEST: $1" | tee -a $TEST_LOG
    echo "----------------------------------------" | tee -a $TEST_LOG
}

cleanup_containers() {
    info "Cleaning up test containers..."
    docker ps -a --filter "name=test-" --format "{{.ID}}" | xargs -r docker rm -f 2>/dev/null || true
    docker ps -a --filter "name=dbarena-" --format "{{.ID}}" | xargs -r docker rm -f 2>/dev/null || true
}

# Start tests
echo "" | tee -a $TEST_LOG
echo "=== PHASE 1: v0.1.0 Core Functionality ===" | tee -a $TEST_LOG

# Test 0.1: Basic PostgreSQL Container Creation
run_test "0.1 - Basic PostgreSQL Container Creation"
if OUTPUT=$($DBARENA create postgres --name test-pg-basic 2>&1); then
    if echo "$OUTPUT" | grep -q "Created successfully"; then
        pass_test "PostgreSQL container created"
    else
        fail_test "PostgreSQL container output invalid" "$OUTPUT"
    fi
else
    fail_test "PostgreSQL container creation failed" "$OUTPUT"
fi

# Test 0.2: List Containers
run_test "0.2 - List Containers"
if OUTPUT=$($DBARENA list 2>&1); then
    if echo "$OUTPUT" | grep -q "test-pg-basic"; then
        pass_test "Container appears in list"
    else
        fail_test "Container not in list" "$OUTPUT"
    fi
else
    fail_test "List command failed" "$OUTPUT"
fi

# Test 0.3: Inspect Container
run_test "0.3 - Inspect Container"
if OUTPUT=$($DBARENA inspect test-pg-basic 2>&1); then
    if echo "$OUTPUT" | grep -q "Database Type:.*postgres"; then
        pass_test "Inspect shows correct database type"
    else
        fail_test "Inspect output incorrect" "$OUTPUT"
    fi
else
    fail_test "Inspect command failed" "$OUTPUT"
fi

# Test 0.4: Stop Container
run_test "0.4 - Stop Container"
if OUTPUT=$($DBARENA stop test-pg-basic 2>&1); then
    sleep 2
    if $DBARENA list --all | grep -q "test-pg-basic.*stopped"; then
        pass_test "Container stopped successfully"
    else
        fail_test "Container not stopped" "$OUTPUT"
    fi
else
    fail_test "Stop command failed" "$OUTPUT"
fi

# Test 0.5: Start Container
run_test "0.5 - Start Container"
if OUTPUT=$($DBARENA start test-pg-basic 2>&1); then
    sleep 2
    if $DBARENA list | grep -q "test-pg-basic"; then
        pass_test "Container started successfully"
    else
        fail_test "Container not started" "$OUTPUT"
    fi
else
    fail_test "Start command failed" "$OUTPUT"
fi

# Test 0.6: Destroy Container
run_test "0.6 - Destroy Container"
if OUTPUT=$($DBARENA destroy test-pg-basic --yes 2>&1); then
    if ! $DBARENA list --all | grep -q "test-pg-basic"; then
        pass_test "Container destroyed successfully"
    else
        fail_test "Container still exists" "$OUTPUT"
    fi
else
    fail_test "Destroy command failed" "$OUTPUT"
fi

echo "" | tee -a $TEST_LOG
echo "=== PHASE 2: v0.2.0 Configuration Management ===" | tee -a $TEST_LOG

# Test 2.1: Config File Creation
run_test "2.1 - Config init command"
if OUTPUT=$($DBARENA config init 2>&1); then
    if echo "$OUTPUT" | grep -q "Example configuration"; then
        pass_test "Config init generates example"
    else
        fail_test "Config init output invalid" "$OUTPUT"
    fi
else
    fail_test "Config init command failed" "$OUTPUT"
fi

# Test 2.2: Create config file
run_test "2.2 - Create test config file"
cat > /tmp/dbarena-test.toml << 'EOF'
[profiles.dev]
[profiles.dev.env]
POSTGRES_DB = "devdb"
POSTGRES_USER = "devuser"
POSTGRES_PASSWORD = "devpass"
EOF

if $DBARENA config validate --config /tmp/dbarena-test.toml 2>&1 | grep -q "valid"; then
    pass_test "Config file validates"
else
    fail_test "Config validation failed"
fi

# Test 2.3: Create container with config profile
run_test "2.3 - Create container with config profile"
if OUTPUT=$($DBARENA create postgres --config /tmp/dbarena-test.toml --profile dev --name test-pg-config 2>&1); then
    if echo "$OUTPUT" | grep -q "Created successfully"; then
        # Verify env vars were applied
        if docker exec test-pg-config env | grep -q "POSTGRES_DB=devdb"; then
            pass_test "Created successfully with config profile"
        else
            fail_test "Environment variables not applied" "$(docker exec test-pg-config env | grep POSTGRES)"
        fi
    else
        fail_test "Container creation with config failed" "$OUTPUT"
    fi
else
    fail_test "Create with config failed" "$OUTPUT"
fi

$DBARENA destroy test-pg-config --yes 2>/dev/null || true

# Test 2.4: CLI override precedence
run_test "2.4 - CLI override precedence"
if OUTPUT=$($DBARENA create postgres --config /tmp/dbarena-test.toml --profile dev --env POSTGRES_USER=cliuser --name test-pg-override 2>&1); then
    if docker exec test-pg-override env | grep -q "POSTGRES_USER=cliuser"; then
        pass_test "CLI override takes precedence"
    else
        fail_test "CLI override not applied" "$(docker exec test-pg-override env | grep POSTGRES_USER)"
    fi
else
    fail_test "Create with override failed" "$OUTPUT"
fi

$DBARENA destroy test-pg-override --yes 2>/dev/null || true

echo "" | tee -a $TEST_LOG
echo "=== PHASE 3: v0.2.0 Init Scripts ===" | tee -a $TEST_LOG

# Test 3.1: Create init script
run_test "3.1 - Init script execution"
cat > /tmp/test-init.sql << 'EOF'
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL
);

INSERT INTO users (username, email) VALUES
    ('alice', 'alice@example.com'),
    ('bob', 'bob@example.com');
EOF

if OUTPUT=$($DBARENA create postgres --name test-pg-init --init-script /tmp/test-init.sql 2>&1); then
    sleep 3
    # Verify table was created
    if docker exec test-pg-init psql -U postgres -d postgres -c "SELECT COUNT(*) FROM users;" 2>/dev/null | grep -q "2"; then
        pass_test "Init script executed successfully"
    else
        fail_test "Init script did not create table/data" "$(docker exec test-pg-init psql -U postgres -d postgres -c 'SELECT * FROM users;' 2>&1)"
    fi
else
    fail_test "Create with init script failed" "$OUTPUT"
fi

$DBARENA destroy test-pg-init --yes 2>/dev/null || true

# Test 3.2: Init script error handling
run_test "3.2 - Init script error handling"
cat > /tmp/test-error.sql << 'EOF'
CREATE TABLE bad_table (
    id SERIAL PRIMARY KEY
    missing_comma VARCHAR(50)
);
EOF

if OUTPUT=$($DBARENA create postgres --name test-pg-error --init-script /tmp/test-error.sql --keep-on-error 2>&1); then
    fail_test "Should have failed on error script" "$OUTPUT"
else
    if echo "$OUTPUT" | grep -q "syntax error"; then
        pass_test "Init script error detected and reported"
    else
        fail_test "Error not properly reported" "$OUTPUT"
    fi
fi

docker rm -f test-pg-error 2>/dev/null || true

echo "" | tee -a $TEST_LOG
echo "=== PHASE 4: v0.2.0 Exec Command ===" | tee -a $TEST_LOG

# Test 4.1: Create test container for exec
$DBARENA create postgres --name test-pg-exec 2>&1 >/dev/null
sleep 3

# Test 4.2: Exec inline SQL
run_test "4.1 - Exec inline SQL"
if OUTPUT=$($DBARENA exec test-pg-exec "CREATE TABLE exec_test (id INT);" 2>&1); then
    if docker exec test-pg-exec psql -U postgres -d postgres -c "\dt" 2>/dev/null | grep -q "exec_test"; then
        pass_test "Exec inline SQL succeeded"
    else
        fail_test "Table not created" "$(docker exec test-pg-exec psql -U postgres -d postgres -c '\dt' 2>&1)"
    fi
else
    fail_test "Exec command failed" "$OUTPUT"
fi

# Test 4.3: Exec from file
run_test "4.2 - Exec from file"
echo "INSERT INTO exec_test (id) VALUES (1), (2), (3);" > /tmp/test-exec.sql
if OUTPUT=$($DBARENA exec test-pg-exec --file /tmp/test-exec.sql 2>&1); then
    if docker exec test-pg-exec psql -U postgres -d postgres -c "SELECT COUNT(*) FROM exec_test;" 2>/dev/null | grep -q "3"; then
        pass_test "Exec from file succeeded"
    else
        fail_test "Data not inserted" "$(docker exec test-pg-exec psql -U postgres -d postgres -c 'SELECT * FROM exec_test;' 2>&1)"
    fi
else
    fail_test "Exec from file failed" "$OUTPUT"
fi

$DBARENA destroy test-pg-exec --yes 2>/dev/null || true

echo "" | tee -a $TEST_LOG
echo "=== PHASE 5: Backwards Compatibility ===" | tee -a $TEST_LOG

# Test 5.1: v0.1.0 commands work without config
run_test "5.1 - v0.1.0 create without config"
if OUTPUT=$($DBARENA create mysql --name test-mysql-compat 2>&1); then
    if echo "$OUTPUT" | grep -q "Created successfully"; then
        pass_test "v0.1.0 create works without config"
    else
        fail_test "Create output invalid" "$OUTPUT"
    fi
else
    fail_test "v0.1.0 create failed" "$OUTPUT"
fi

# Test 5.2: All v0.1.0 flags still work
run_test "5.2 - v0.1.0 flags still work"
if OUTPUT=$($DBARENA create postgres --name test-pg-flags --port 54321 --memory 512 2>&1); then
    if docker inspect test-pg-flags --format '{{.HostConfig.PortBindings}}' | grep -q "54321"; then
        pass_test "v0.1.0 flags work correctly"
    else
        fail_test "Port flag not applied" "$(docker inspect test-pg-flags --format '{{.HostConfig.PortBindings}}')"
    fi
else
    fail_test "Create with flags failed" "$OUTPUT"
fi

$DBARENA destroy test-mysql-compat --yes 2>/dev/null || true
$DBARENA destroy test-pg-flags --yes 2>/dev/null || true

# Cleanup
cleanup_containers

# Summary
echo "" | tee -a $TEST_LOG
echo "================================================================" | tee -a $TEST_LOG
echo "TEST SUMMARY" | tee -a $TEST_LOG
echo "================================================================" | tee -a $TEST_LOG
echo -e "Total Tests: $((PASSED + FAILED))" | tee -a $TEST_LOG
echo -e "${GREEN}Passed: $PASSED${NC}" | tee -a $TEST_LOG
echo -e "${RED}Failed: $FAILED${NC}" | tee -a $TEST_LOG

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}✓ ALL TESTS PASSED!${NC}" | tee -a $TEST_LOG
    exit 0
else
    echo -e "\n${RED}✗ SOME TESTS FAILED${NC}" | tee -a $TEST_LOG
    exit 1
fi
