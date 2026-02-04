# Workload Generation Guide

This guide covers how to use dbarena's workload generation capabilities to simulate realistic database transactions and measure performance.

## Table of Contents
- [Quick Start](#quick-start)
- [Built-in Patterns](#built-in-patterns)
- [Configuration Format](#configuration-format)
- [Custom Workloads](#custom-workloads)
- [Live Monitoring](#live-monitoring)
- [Performance Metrics](#performance-metrics)
- [Examples](#examples)
- [Best Practices](#best-practices)

## Quick Start

### Basic Workflow

```bash
# 1. Create and seed a database
dbarena create postgres --name mydb
dbarena seed --config seed.toml --container mydb

# 2. Run a workload with a built-in pattern
dbarena workload run \
    --container mydb \
    --pattern oltp \
    --tps 100 \
    --duration 60

# 3. Monitor in real-time (optional)
dbarena stats --multipane
```

### Simple Example

```bash
# Run balanced workload for 30 seconds at 50 TPS
dbarena workload run \
    --container mydb \
    --pattern balanced \
    --tps 50 \
    --duration 30
```

Output:
```
======================================================================
Workload Progress
======================================================================

  ⏱ Time: 15.3s / 30.0s (51.0%)
     [=========================                         ]

  ⚡ TPS: 49.8 (target: 50)
     ✓ On target

  ✓ Success: 98.5% (736 / 747 total)

  ⏲ Latency:
     P50: 12.34ms
     P95: 45.67ms
     P99: 78.90ms
     Mean: 23.45ms

  📝 Operations:
     SELECT: 373 (49.9%)
     INSERT: 187 (25.0%)
     UPDATE: 149 (19.9%)
     DELETE: 38 (5.1%)
```

## Built-in Patterns

dbarena includes 10 built-in workload patterns covering common scenarios:

### 1. OLTP (Online Transaction Processing)
**Use Case:** Banking, booking systems, general transactional workloads

**Characteristics:**
- 40% SELECT, 30% INSERT, 25% UPDATE, 5% DELETE
- Small transactions
- Frequent commits
- Primary key lookups

```bash
dbarena workload run --container mydb --pattern oltp --tps 200 --duration 300
```

### 2. E-commerce
**Use Case:** Shopping carts, orders, inventory management

**Characteristics:**
- 50% SELECT (browsing), 25% INSERT (orders), 20% UPDATE (inventory/cart), 5% DELETE
- Mix of reads and writes
- Complex transactions

```bash
dbarena workload run --container mydb --pattern ecommerce --tps 100 --duration 180
```

### 3. OLAP (Online Analytical Processing)
**Use Case:** Data warehousing, business intelligence

**Characteristics:**
- 90% SELECT with JOINs and aggregations
- 5% INSERT, 4% UPDATE, 1% DELETE
- Complex queries, large result sets

```bash
dbarena workload run --container mydb --pattern olap --tps 10 --duration 600
```

### 4. Reporting
**Use Case:** Dashboards, report generation

**Characteristics:**
- 95% SELECT, mostly read-only
- Large result sets
- Aggregations and GROUP BY
- Minimal writes

```bash
dbarena workload run --container mydb --pattern reporting --tps 20 --duration 300
```

### 5. Time-Series
**Use Case:** IoT, sensor data, logs

**Characteristics:**
- 30% SELECT (range queries), 65% INSERT (sensor data), 2% UPDATE, 3% DELETE
- Append-heavy
- Time-based queries
- High insert rate

```bash
dbarena workload run --container mydb --pattern time_series --tps 500 --duration 600
```

### 6. Social Media
**Use Case:** Social networks, feeds, timelines

**Characteristics:**
- 70% SELECT (feeds/timelines), 20% INSERT (posts), 8% UPDATE (likes), 2% DELETE
- High read volume
- Burst writes
- Complex JOINs

```bash
dbarena workload run --container mydb --pattern social_media --tps 300 --duration 300
```

### 7. IoT
**Use Case:** IoT platforms, telemetry

**Characteristics:**
- 20% SELECT (aggregations), 75% INSERT (sensor ingestion), 3% UPDATE, 2% DELETE
- Very high insert rate
- Time-series aggregations

```bash
dbarena workload run --container mydb --pattern iot --tps 1000 --duration 300
```

### 8. Read-Heavy
**Use Case:** Caching, content delivery

**Characteristics:**
- 80% SELECT, 10% INSERT, 8% UPDATE, 2% DELETE
- Simple queries
- High read throughput

```bash
dbarena workload run --container mydb --pattern read_heavy --tps 500 --duration 180
```

### 9. Write-Heavy
**Use Case:** Logging, event sourcing

**Characteristics:**
- 20% SELECT, 40% INSERT, 30% UPDATE, 10% DELETE
- High write throughput
- Simple operations

```bash
dbarena workload run --container mydb --pattern write_heavy --tps 300 --duration 180
```

### 10. Balanced
**Use Case:** General purpose, mixed workload

**Characteristics:**
- 50% SELECT, 25% INSERT, 20% UPDATE, 5% DELETE
- Even mix of operations

```bash
dbarena workload run --container mydb --pattern balanced --tps 100 --duration 120
```

## Configuration Format

### Basic Configuration

```toml
name = "My Custom Workload"
tables = ["users", "orders", "products"]
connections = 20           # Concurrent workers
target_tps = 100          # Target transactions per second
duration_seconds = 300    # Run for 5 minutes
```

### Pattern-Based Configuration

```toml
name = "E-commerce Load Test"
pattern = "ecommerce"     # Use built-in pattern
tables = ["users", "orders", "products", "order_items"]
connections = 50
target_tps = 200
duration_seconds = 600
```

### Transaction Count Instead of Duration

```toml
name = "Fixed Transaction Count"
pattern = "oltp"
tables = ["users", "orders"]
connections = 10
target_tps = 50
transaction_count = 10000  # Run until 10K transactions
```

## Custom Workloads

### Method 1: Custom Operation Mix

Define custom operation weights:

```toml
name = "Custom Analytics Workload"
tables = ["orders", "customers", "products"]
connections = 20
target_tps = 100
duration_seconds = 300

[custom_operations]
select_weight = 0.85      # 85% SELECT
insert_weight = 0.05      # 5% INSERT
update_weight = 0.08      # 8% UPDATE
delete_weight = 0.02      # 2% DELETE
use_joins = true          # Include JOIN queries
use_aggregations = true   # Include SUM/COUNT/AVG
avg_result_set_size = 1000  # Expected result size
```

**Weights must sum to 1.0**

### Method 2: Custom SQL Queries

Define specific SQL queries with parameters:

```toml
name = "Custom SQL Workload"
tables = ["orders", "customers"]
connections = 10
target_tps = 50
duration_seconds = 120

# Query 1: Recent orders by customer
[[custom_queries]]
name = "recent_orders"
sql = "SELECT * FROM orders WHERE customer_id = :customer_id AND created_at > NOW() - INTERVAL '1 hour' ORDER BY created_at DESC"
weight = 0.4  # 40% of transactions

[[custom_queries.parameters]]
name = "customer_id"
generator = "foreign_key"
[custom_queries.parameters.options]
references = { table = "customers", column = "id" }

# Query 2: Create order
[[custom_queries]]
name = "create_order"
sql = "INSERT INTO orders (customer_id, total, status) VALUES (:customer_id, :total, 'pending')"
weight = 0.3  # 30% of transactions

[[custom_queries.parameters]]
name = "customer_id"
generator = "foreign_key"
[custom_queries.parameters.options]
references = { table = "customers", column = "id" }

[[custom_queries.parameters]]
name = "total"
generator = "random_decimal"
[custom_queries.parameters.options]
min = 10.0
max = 1000.0
precision = 2

# Query 3: Update order status
[[custom_queries]]
name = "update_order_status"
sql = "UPDATE orders SET status = :status WHERE id = :order_id"
weight = 0.3  # 30% of transactions

[[custom_queries.parameters]]
name = "order_id"
generator = "foreign_key"
[custom_queries.parameters.options]
references = { table = "orders", column = "id" }

[[custom_queries.parameters]]
name = "status"
generator = "enum"
[custom_queries.parameters.options]
values = ["pending", "processing", "shipped", "delivered"]
```

**Weights must sum to 1.0**

### Query Parameter Generators

Use data generators for query parameters:

**Foreign Key:**
```toml
[[custom_queries.parameters]]
name = "user_id"
generator = "foreign_key"
[custom_queries.parameters.options]
references = { table = "users", column = "id" }
```

**Random Decimal:**
```toml
[[custom_queries.parameters]]
name = "price"
generator = "random_decimal"
[custom_queries.parameters.options]
min = 10.0
max = 500.0
precision = 2
```

**Enum:**
```toml
[[custom_queries.parameters]]
name = "status"
generator = "enum"
[custom_queries.parameters.options]
values = ["active", "inactive", "pending"]
```

**Template:**
```toml
[[custom_queries.parameters]]
name = "description"
generator = "template"
[custom_queries.parameters.options]
template = "Transaction {random_int:1000:9999}"
```

## Live Monitoring

### Progress Display

While a workload runs, dbarena shows live progress:

```
======================================================================
Workload Progress
======================================================================

  ⏱ Time: 42.7s / 60.0s (71.2%)
     [===================================               ]

  ⚡ TPS: 98.3 (target: 100)
     ✓ On target

  ✓ Success: 99.2% (4198 / 4232 total)
  ✗ Failed: 34 transactions

  ⏲ Latency:
     P50: 8.45ms
     P95: 32.10ms
     P99: 67.89ms
     Mean: 15.23ms

  📝 Operations:
     SELECT: 2116 (50.0%)
     INSERT: 1058 (25.0%)
     UPDATE: 846 (20.0%)
     DELETE: 212 (5.0%)

  ⚠ Recent Errors:
     3: connection timeout
     1: deadlock detected
```

Updates every second.

### Concurrent Monitoring

Monitor system and database metrics while workload runs:

```bash
# Terminal 1: Run workload
dbarena workload run --container mydb --pattern oltp --tps 500 --duration 300

# Terminal 2: Monitor metrics
dbarena stats --multipane
```

The TUI shows:
- CPU/memory usage
- Database QPS/TPS
- Workload activity
- Latency metrics

## Performance Metrics

### Final Summary

After workload completes:

```
======================================================================
Workload Complete
======================================================================

  Pattern: OLTP Workload
  Duration: 60.02s

  📊 Total Transactions: 6003
     Successful: 5989 (99.8%)
     Failed: 14

  ⚡ Throughput: 99.9 TPS

  ⏲ Latency:
     P50: 9.12ms
     P95: 35.67ms
     P99: 72.45ms
     Max: 156.23ms

  📝 Operation Distribution:
     SELECT: 2996 (49.9%)
     INSERT: 1501 (25.0%)
     UPDATE: 1201 (20.0%)
     DELETE: 305 (5.1%)

  ⚠ Error Summary:
     Total errors: 14
     Unique error types: 2
```

### Key Metrics

**TPS (Transactions Per Second):**
- Actual throughput achieved
- Compare to target TPS
- ±10% accuracy expected

**Latency Percentiles:**
- **P50 (median)**: 50% of transactions faster than this
- **P95**: 95% of transactions faster than this
- **P99**: 99% of transactions faster than this
- **Max**: Slowest transaction

**Success Rate:**
- Percentage of successful transactions
- >95% indicates healthy system
- <95% may indicate overload or issues

**Operation Distribution:**
- Breakdown by operation type
- Should match configured weights
- Useful for understanding load characteristics

## Examples

### Example 1: E-commerce Performance Test

**Goal:** Test an e-commerce database under realistic load

**Setup:**
```bash
# 1. Seed database
dbarena seed --config seed-ecommerce.toml --container mydb --size medium

# 2. Create workload config
cat > workload-ecommerce.toml << 'EOF'
name = "E-commerce Load Test"
pattern = "ecommerce"
tables = ["users", "products", "orders", "order_items"]
connections = 50
target_tps = 200
duration_seconds = 600  # 10 minutes
EOF

# 3. Run workload
dbarena workload run --config workload-ecommerce.toml --container mydb
```

**Expected Results:**
- TPS: 180-220 (±10% of 200)
- P99 latency: <100ms
- Success rate: >95%

### Example 2: IoT Sensor Ingestion Test

**Goal:** Test high-volume sensor data ingestion

```bash
# High TPS for sensor data
dbarena workload run \
    --container iot-db \
    --pattern iot \
    --tps 1000 \
    --duration 300 \
    --connections 100
```

**Characteristics:**
- Very high INSERT rate (75% of operations)
- Many concurrent connections
- Sustained load over 5 minutes

### Example 3: Comparing Database Performance

**Goal:** Compare PostgreSQL vs MySQL performance

```bash
# Test PostgreSQL
dbarena create postgres --name pg-test
dbarena seed --config seed.toml --container pg-test --seed 42
dbarena workload run --container pg-test --pattern oltp --tps 500 --duration 300

# Test MySQL (identical data and workload)
dbarena create mysql --name mysql-test
dbarena seed --config seed.toml --container mysql-test --seed 42
dbarena workload run --container mysql-test --pattern oltp --tps 500 --duration 300

# Compare results
```

### Example 4: Stress Testing

**Goal:** Find maximum TPS capacity

```bash
# Start at baseline
dbarena workload run --container mydb --pattern oltp --tps 100 --duration 60

# Increase gradually
dbarena workload run --container mydb --pattern oltp --tps 200 --duration 60
dbarena workload run --container mydb --pattern oltp --tps 500 --duration 60
dbarena workload run --container mydb --pattern oltp --tps 1000 --duration 60

# Stop when success rate drops below 95% or P99 > 100ms
```

## Best Practices

### 1. Start with Built-in Patterns
Use built-in patterns before creating custom workloads:

```bash
# Good: Start simple
dbarena workload run --container mydb --pattern balanced --tps 100 --duration 60

# Then customize if needed
```

### 2. Match TPS to Your Use Case
- **Development/Testing:** 10-100 TPS
- **Staging:** 100-500 TPS
- **Production Simulation:** 500-1000+ TPS

### 3. Monitor During Long Runs
For workloads >5 minutes, monitor in real-time:

```bash
# Terminal 1
dbarena workload run --container mydb --pattern oltp --tps 500 --duration 3600

# Terminal 2
dbarena stats --multipane
```

### 4. Use Realistic Concurrency
Match connection count to your application:
- **Web Apps:** 10-50 connections
- **Microservices:** 50-200 connections
- **High-Scale Systems:** 200-500+ connections

### 5. Validate Before Long Runs
Test with short duration first:

```bash
# Test for 30 seconds
dbarena workload run --container mydb --pattern oltp --tps 500 --duration 30

# If successful, run full test
dbarena workload run --container mydb --pattern oltp --tps 500 --duration 3600
```

### 6. Use Deterministic Seeds
For reproducible results, seed with same value:

```bash
dbarena seed --config seed.toml --container mydb --seed 42
dbarena workload run --container mydb --pattern oltp --tps 100 --duration 60
```

### 7. Track Metrics Over Time
Export results for comparison:

```bash
dbarena workload run --container mydb --pattern oltp --tps 500 --duration 300 > run1.txt
# Make changes
dbarena workload run --container mydb --pattern oltp --tps 500 --duration 300 > run2.txt
# Compare run1.txt vs run2.txt
```

### 8. Combine with CDC Testing
Use workload generation to test CDC systems:

```bash
# 1. Enable CDC (v0.6.0)
dbarena cdc enable --container mydb

# 2. Run workload to generate changes
dbarena workload run --container mydb --pattern ecommerce --tps 100 --duration 300

# 3. Monitor CDC (v0.7.0)
dbarena cdc monitor --container mydb
```

## Troubleshooting

### Low TPS (Below Target)
**Causes:**
- Database overloaded
- Insufficient connections
- Slow queries

**Solutions:**
- Reduce target TPS
- Increase connections
- Optimize database (indexes, config)
- Use simpler pattern (e.g., read_heavy)

### High Latency
**Causes:**
- Database under heavy load
- Complex queries
- Resource constraints

**Solutions:**
- Reduce TPS or connections
- Simplify workload pattern
- Increase database resources (memory, CPU)
- Add database indexes

### Low Success Rate (<95%)
**Causes:**
- Database errors (deadlocks, timeouts)
- Overloaded system
- Constraint violations

**Solutions:**
- Check error messages in progress display
- Reduce TPS or connections
- Fix database issues (deadlocks, slow queries)
- Verify seeded data is valid

### Inconsistent Results
**Causes:**
- Variable database state
- Background processes
- Resource contention

**Solutions:**
- Use deterministic seeding (`--seed 42`)
- Restart database between runs
- Run longer tests (300+ seconds) for stable averages
- Isolate test environment

## Next Steps

- See [Data Seeding Guide](seeding.md) to populate test data
- See [Testing Guide](TESTING_PHASE8.md) for performance benchmarks
- Check [examples/](../examples/) for more workload configurations
- Read [v0.5.0 spec](../specs/v0.5.0/VERSION_OVERVIEW.md) for technical details
