# Phase 8: Performance & Integration Testing Guide

This document describes the comprehensive testing strategy for v0.5.0 (Data Seeding + Workload Generation).

## Test Overview

Phase 8 includes three categories of tests:
1. **Performance Tests** - Validate NFR targets
2. **Scale Tests** - Verify stability at scale
3. **Cross-Database Tests** - Ensure compatibility

## Test Prerequisites

### Docker Environment
```bash
# Verify Docker is running
docker ps

# Ensure sufficient resources:
# - Memory: 4GB+ available
# - Disk: 10GB+ free
# - CPU: 4+ cores recommended
```

### Build the Project
```bash
cd /Users/timdelisle/Dev/db-simulation-env/cli
cargo build --release
```

## Running Tests

### Unit Tests (Fast)
```bash
# Run all unit tests
cargo test --lib

# Expected: 121 tests pass in ~2s
```

### Performance Tests (Slow - Run Individually)

#### Test 1: Seeding Performance (100K rows in <60s)
```bash
cargo test --test phase8_performance test_seeding_performance_100k_rows -- --ignored --nocapture

# Expected output:
# ✅ Seeded 100000 rows in ~30-50s
#    Rate: ~2000-3000 rows/sec
#
# NFR Target: <60 seconds
# Status: PASS if elapsed < 60s
```

#### Test 2: Workload TPS Accuracy (±10%)
```bash
cargo test --test phase8_performance test_workload_tps_accuracy -- --ignored --nocapture

# Expected output:
# ✅ Achieved ~100 TPS (target: 100 TPS)
#    Duration: ~30s
#    Total transactions: ~3000
#    Success rate: >95%
#
# NFR Target: Within ±10% of target TPS
# Status: PASS if TPS diff ≤ 10%
```

#### Test 3: Latency P99 <100ms
```bash
cargo test --test phase8_performance test_workload_latency_p99 -- --ignored --nocapture

# Expected output:
# ✅ Latency metrics:
#    P50: ~5-15ms
#    P95: ~20-40ms
#    P99: ~40-80ms
#    Max: ~100-200ms
#
# NFR Target: P99 <100ms under normal load
# Status: PASS if P99 < 100ms
```

### Scale Tests (Very Slow - Optional)

#### Test 4: Large Scale Seeding (1M rows)
```bash
cargo test --test phase8_performance test_seeding_scale_1m_rows -- --ignored --nocapture

# Expected output:
# ✅ Seeded 1000000 rows in ~300-600s
#    Rate: ~1500-3000 rows/sec
#
# Target: Completes without errors
# Status: PASS if no panics/errors
```

#### Test 5: Long Duration Workload (5 minutes)
```bash
cargo test --test phase8_performance test_workload_long_duration -- --ignored --nocapture

# Expected output:
# ✅ Completed 5-minute workload:
#    Total transactions: ~150,000
#    Success rate: >95%
#    Average TPS: ~500
#    Duration: 300s
#
# Target: Maintains stability over time
# Status: PASS if success rate ≥95%
```

## Manual End-to-End Testing

### E2E Test 1: Complete Seeding Workflow

```bash
# 1. Create test container
./target/release/dbarena create postgres --name e2e-seed-test

# 2. Create test schema
cat > /tmp/test_schema.sql << 'EOF'
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255),
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    total DECIMAL(10, 2),
    status VARCHAR(50),
    created_at TIMESTAMP DEFAULT NOW()
);
EOF

./target/release/dbarena exec e2e-seed-test -- psql -U postgres -f /tmp/test_schema.sql

# 3. Create seed config
cat > /tmp/seed_config.toml << 'EOF'
[seed_rules]
global_seed = 42

[[seed_rules.tables]]
name = "users"
count = 1000

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"
[seed_rules.tables.columns.options]
start = 1

[[seed_rules.tables.columns]]
name = "email"
generator = "email"

[[seed_rules.tables.columns]]
name = "name"
generator = "name"
[seed_rules.tables.columns.options]
type = "full"

[[seed_rules.tables]]
name = "orders"
count = 5000

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"

[[seed_rules.tables.columns]]
name = "user_id"
generator = "foreign_key"
[seed_rules.tables.columns.options]
references = { table = "users", column = "id" }

[[seed_rules.tables.columns]]
name = "total"
generator = "random_decimal"
[seed_rules.tables.columns.options]
min = 10.0
max = 1000.0
precision = 2

[[seed_rules.tables.columns]]
name = "status"
generator = "enum"
[seed_rules.tables.columns.options]
values = ["pending", "processing", "shipped", "delivered"]
EOF

# 4. Run seeding
./target/release/dbarena seed --config /tmp/seed_config.toml --container e2e-seed-test

# Expected:
# - 1000 users seeded
# - 5000 orders seeded with valid user_id references
# - Completes in <10 seconds

# 5. Verify data
./target/release/dbarena exec e2e-seed-test -- psql -U postgres -c "SELECT COUNT(*) FROM users"
# Expected: 1000

./target/release/dbarena exec e2e-seed-test -- psql -U postgres -c "SELECT COUNT(*) FROM orders"
# Expected: 5000

./target/release/dbarena exec e2e-seed-test -- psql -U postgres -c "SELECT COUNT(*) FROM orders WHERE user_id NOT IN (SELECT id FROM users)"
# Expected: 0 (no FK violations)

# 6. Cleanup
./target/release/dbarena destroy e2e-seed-test -y
```

### E2E Test 2: Complete Workload Workflow

```bash
# 1. Create and setup container (reuse steps 1-2 from E2E Test 1)
./target/release/dbarena create postgres --name e2e-workload-test

# Setup schema...

# 2. Seed data (reuse step 3-4 from E2E Test 1)
./target/release/dbarena seed --config /tmp/seed_config.toml --container e2e-workload-test

# 3. Run workload with built-in pattern
./target/release/dbarena workload run \
    --container e2e-workload-test \
    --pattern oltp \
    --tps 100 \
    --duration 30

# Expected:
# - Live progress display showing TPS, latency, success rate
# - Final summary with metrics
# - TPS within ±10% of 100
# - P99 latency <100ms
# - Success rate >95%

# 4. Create custom workload config
cat > /tmp/workload_config.toml << 'EOF'
name = "E2E Custom Workload"
tables = ["users", "orders"]
connections = 20
target_tps = 50
duration_seconds = 60

[custom_operations]
select_weight = 0.5
insert_weight = 0.2
update_weight = 0.2
delete_weight = 0.1
EOF

# 5. Run custom workload
./target/release/dbarena workload run \
    --container e2e-workload-test \
    --config /tmp/workload_config.toml

# 6. Cleanup
./target/release/dbarena destroy e2e-workload-test -y
```

## Performance Optimization Notes

Based on test results, apply these optimizations if needed:

### Seeding Performance

**If seeding is slower than expected:**

1. **Increase batch size:**
   - Current default: 1,000 rows/batch
   - Try: 5,000 or 10,000 rows/batch
   - Location: `SeedingEngine::new()` batch_size parameter

2. **Use database-specific bulk load:**
   - Postgres: Use COPY instead of INSERT
   - MySQL: Use LOAD DATA INFILE
   - SQL Server: Use BULK INSERT
   - Location: `src/seed/sql_builder.rs`

3. **Disable indexes during bulk load:**
   - Drop indexes before seeding
   - Rebuild after seeding completes
   - Significant speedup for large datasets

4. **Parallel table seeding:**
   - Already implemented for independent tables
   - Verify dependency resolver correctly identifies parallel opportunities
   - Location: `src/seed/dependency.rs`

### Workload Performance

**If TPS is lower than expected:**

1. **Increase connection pool size:**
   - More workers = higher throughput
   - Trade-off: more database connections
   - Configure via `connections` parameter

2. **Reduce operation complexity:**
   - Simple SELECT by ID is fastest
   - JOINs and aggregations are slower
   - Adjust pattern weights in config

3. **Optimize rate limiter:**
   - Governor crate uses token bucket algorithm
   - Burst capacity can be tuned
   - Location: `src/workload/rate_limiter.rs`

**If latency is higher than expected:**

1. **Check database resource allocation:**
   - Ensure sufficient memory
   - Check CPU limits
   - Monitor disk I/O

2. **Reduce concurrent load:**
   - Fewer connections = lower contention
   - Lower TPS = lower latency

3. **Optimize SQL queries:**
   - Add indexes on frequently queried columns
   - Analyze query plans
   - Consider connection pooling overhead

## Success Criteria Checklist

### Phase 8 Complete When:

- [ ] All unit tests pass (121 tests)
- [ ] Seeding performance test passes (<60s for 100K rows)
- [ ] Workload TPS accuracy test passes (±10%)
- [ ] Latency test passes (P99 <100ms)
- [ ] Scale tests complete without errors
- [ ] E2E manual tests work as documented
- [ ] Documentation complete and accurate

### NFR Summary

| Metric | Target | Test |
|--------|--------|------|
| Seeding: 100K rows | <60 seconds | test_seeding_performance_100k_rows |
| Workload TPS accuracy | ±10% of target | test_workload_tps_accuracy |
| Latency P99 | <100ms | test_workload_latency_p99 |
| Long workload success | >95% | test_workload_long_duration |
| Scale: 1M rows | No errors | test_seeding_scale_1m_rows |

## Next Steps

After Phase 8 completion:
1. Update specifications (Phase 9)
2. Create user documentation (Phase 10)
3. Prepare v0.5.0 release
