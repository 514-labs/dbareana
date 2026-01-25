#!/bin/bash
# Quick Manual Tests for dbarena v0.2.0 - Critical tests only
set -e

DBARENA="./target/release/dbarena"
PASSED=0
FAILED=0

pass_test() {
    echo "✓ PASS: $1"
    ((PASSED++))
}

fail_test() {
    echo "✗ FAIL: $1"
    echo "  Error: $2"
    ((FAILED++))
}

info() {
    echo "ℹ INFO: $1"
}

echo "================================================================"
echo "dbarena v0.2.0 Quick Manual Tests"
echo "================================================================"
echo ""

# Test 1: Basic create
echo "TEST 1: Basic container creation"
if $DBARENA create postgres --name quick-test-pg 2>&1 | grep -q "Created successfully"; then
    pass_test "PostgreSQL container created"
else
    fail_test "PostgreSQL container creation failed"
fi

# Test 2: List
echo "TEST 2: List containers"
if $DBARENA list | grep -q "quick-test-pg"; then
    pass_test "Container appears in list"
else
    fail_test "Container not in list"
fi

# Test 3: Config file with profile
echo "TEST 3: Config file with profile"
cat > /tmp/test-config.toml << 'EOF'
[profiles.test]
[profiles.test.env]
POSTGRES_DB = "testdb"
POSTGRES_USER = "testuser"
EOF

if $DBARENA create postgres --config /tmp/test-config.toml --profile test --name quick-test-config 2>&1 | grep -q "Created successfully"; then
    if docker exec quick-test-config env | grep -q "POSTGRES_USER=testuser"; then
        pass_test "Config profile applied correctly"
    else
        fail_test "Config env vars not applied"
    fi
else
    fail_test "Create with config failed"
fi

# Test 4: Init script
echo "TEST 4: Init script execution"
cat > /tmp/test-init.sql << 'EOF'
CREATE TABLE test_table (id INT);
INSERT INTO test_table VALUES (1), (2);
EOF

if $DBARENA create postgres --name quick-test-init --init-script /tmp/test-init.sql 2>&1 | grep -q "Created successfully"; then
    sleep 2
    if docker exec quick-test-init psql -U postgres -d postgres -c "SELECT COUNT(*) FROM test_table;" 2>/dev/null | grep -q "2"; then
        pass_test "Init script executed successfully"
    else
        fail_test "Init script did not execute properly"
    fi
else
    fail_test "Create with init script failed"
fi

# Test 5: Exec command
echo "TEST 5: Exec command"
if $DBARENA exec quick-test-pg "CREATE TABLE exec_test (id INT);" 2>&1 > /dev/null; then
    if docker exec quick-test-pg psql -U postgres -d postgres -c "\dt" 2>/dev/null | grep -q "exec_test"; then
        pass_test "Exec command executed successfully"
    else
        fail_test "Exec did not create table"
    fi
else
    fail_test "Exec command failed"
fi

# Test 6: Backwards compatibility
echo "TEST 6: Backwards compatibility (v0.1.0 without config)"
if $DBARENA create mysql --name quick-test-mysql 2>&1 | grep -q "Created successfully"; then
    pass_test "v0.1.0 create works without config"
else
    fail_test "v0.1.0 compatibility broken"
fi

# Cleanup
info "Cleaning up test containers..."
$DBARENA destroy quick-test-pg --yes >/dev/null 2>&1 || true
$DBARENA destroy quick-test-config --yes >/dev/null 2>&1 || true
$DBARENA destroy quick-test-init --yes >/dev/null 2>&1 || true
$DBARENA destroy quick-test-mysql --yes >/dev/null 2>&1 || true

echo ""
echo "================================================================"
echo "SUMMARY"
echo "================================================================"
echo "Total Tests: $((PASSED + FAILED))"
echo "Passed: $PASSED"
echo "Failed: $FAILED"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo "✓ ALL TESTS PASSED!"
    exit 0
else
    echo ""
    echo "✗ SOME TESTS FAILED"
    exit 1
fi
