# Workload Engine Testing Guide

This guide shows how to test the workload functionality that's been implemented so far.

## What's Implemented (Phase 5)

✅ **Core Engine**
- Concurrent worker architecture
- Rate limiting (TPS control)
- Statistics collection with histogram
- Built-in workload patterns (OLTP, E-commerce, etc.)

⏳ **Not Yet Implemented**
- Realistic CRUD operations (Phase 6)
- CLI command `dbarena workload` (Phase 7)

**Current Limitation**: The workload engine executes simple placeholder SQL queries like:
- `SELECT * FROM table LIMIT 10`
- `INSERT INTO table DEFAULT VALUES`
- etc.

Phase 6 will add realistic, data-driven CRUD operations.

## Setup for Testing

### 1. Create a Test Database Container

```bash
# Create a Postgres container for testing
cargo run -- create postgres --name workload-test

# Verify it's running
cargo run -- list
```

### 2. Run Integration Tests

```bash
# Run all workload integration tests
cargo test --test workload_integration_test -- --ignored --nocapture

# Run specific test
cargo test test_workload_engine_end_to_end -- --ignored --nocapture

# Test different patterns
cargo test test_workload_patterns -- --ignored --nocapture

# Test rate limiting accuracy
cargo test test_rate_limiting_accuracy -- --ignored --nocapture

# Test concurrent workers
cargo test test_concurrent_workers -- --ignored --nocapture
```

## What the Tests Verify

### 1. End-to-End Workload Execution
- ✅ Workers spawn correctly
- ✅ Rate limiting enforces TPS
- ✅ Statistics are collected
- ✅ Operations execute via Docker

### 2. Workload Patterns
- ✅ ReadHeavy (80% SELECT)
- ✅ WriteHeavy (80% writes)
- ✅ OLTP (balanced transactional)
- ✅ All 10 patterns work

### 3. Rate Limiting
- ✅ Actual TPS within ±10% of target
- ✅ Consistent over time
- ✅ Works with concurrent workers

### 4. Concurrency
- ✅ 1-20 workers execute correctly
- ✅ No race conditions
- ✅ Stats aggregation is thread-safe

## Expected Output

When you run the end-to-end test, you should see:

```
Starting workload engine test...
Running 5 workers at 50 TPS for 2 seconds

=== Workload Test Results ===
Total transactions: 100
Successful: 95
Failed: 5
Success rate: 95.00%
Duration: 2.01s
TPS: 49.75

P50 latency: 15.23ms
P95 latency: 45.67ms
P99 latency: 78.91ms

Operation counts:
  SELECT: 50
  INSERT: 25
  UPDATE: 20
  DELETE: 5

✓ All assertions passed!
```

## Interpreting Results

### Success Metrics
- **Total transactions**: Should be close to TPS × duration
- **Success rate**: Should be >90% (some failures expected with placeholder SQL)
- **TPS**: Should be within ±10% of target
- **Latency**: P99 should be <100ms for local Docker containers

### Common Issues

**No transactions executed:**
- Check container is running: `cargo run -- list`
- Check container name matches: `workload-test`

**Low success rate (<50%):**
- Expected with placeholder SQL in Phase 5
- Will improve with realistic operations in Phase 6

**TPS way off target:**
- Check system isn't overloaded
- Try reducing worker count or TPS

## Manual Testing (Without CLI)

You can also test programmatically in Rust:

```rust
use dbarena::workload::{WorkloadConfig, WorkloadEngine, WorkloadPattern};
use dbarena::container::DatabaseType;
use std::sync::Arc;
use bollard::Docker;

#[tokio::main]
async fn main() {
    let docker = Arc::new(Docker::connect_with_local_defaults().unwrap());

    let config = WorkloadConfig {
        name: "My Test".to_string(),
        pattern: Some(WorkloadPattern::Balanced),
        custom_operations: None,
        custom_queries: None,
        tables: vec!["pg_database".to_string()],
        connections: 10,
        target_tps: 100,
        duration_seconds: Some(5),
        transaction_count: None,
    };

    let engine = WorkloadEngine::new(
        "workload-test".to_string(),
        DatabaseType::Postgres,
        config,
        docker,
    );

    let stats = engine.run().await.unwrap();

    println!("TPS: {:.2}", stats.tps());
    println!("Success rate: {:.2}%", stats.success_rate());
}
```

## Next Steps

After Phase 6 and 7 are complete, you'll be able to test with:

```bash
# Create and seed a database
dbarena create postgres --name mydb
dbarena seed --config seed.toml --container mydb

# Run realistic workload
dbarena workload run \
  --container mydb \
  --pattern ecommerce \
  --tps 100 \
  --duration 60

# Or with custom config
dbarena workload run \
  --config my-workload.toml \
  --container mydb
```

## Cleanup

```bash
# Stop and remove test container
cargo run -- destroy workload-test -y
```

## Troubleshooting

### "Docker not available"
```bash
# Make sure Docker is running
docker ps
```

### "Container not found: workload-test"
```bash
# Create the test container
cargo run -- create postgres --name workload-test
```

### Tests fail with SQL errors
This is expected in Phase 5 with placeholder SQL. The operations are intentionally simple and some may fail. Phase 6 will add realistic, valid SQL generation.

### Rate limiting off by >20%
- System may be under load
- Try longer duration (5+ seconds)
- Try fewer workers
