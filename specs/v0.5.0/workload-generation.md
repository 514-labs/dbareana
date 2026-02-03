# Workload Generation

## Feature Overview

Workload generation system that simulates realistic database transactions and concurrent operations. Generates CRUD operations against seeded data, supports built-in workload patterns and custom SQL scripts, and provides concurrent connection simulation for load testing and CDC validation.

## Problem Statement

Testing CDC and database performance requires generating database activity:
- Manually running queries is not realistic or scalable
- Writing custom workload generators is time-consuming
- Simulating concurrent connections requires complex application code
- Reproducing specific transaction patterns is difficult
- CDC systems need continuous change streams to test against

Without automated workload generation, users cannot effectively test CDC capture performance or database behavior under load.

## User Stories

**As a CDC developer**, I want to:
- Generate continuous INSERT/UPDATE/DELETE operations to test CDC capture
- Control transaction rate to test CDC under various loads
- Simulate multiple concurrent connections generating changes
- Verify CDC captures all changes without loss

**As a performance engineer**, I want to:
- Run standard OLTP workload patterns (read-heavy, write-heavy)
- Measure database throughput under concurrent load
- Compare performance across database types with identical workloads
- Generate realistic query patterns (not just SELECT * FROM table)

**As a QA engineer**, I want to:
- Create reproducible test workloads
- Run custom SQL scripts as part of test scenarios
- Measure workload success rate and error conditions
- Stop workload after specific duration or transaction count

## Technical Requirements

### Functional Requirements

**FR-1: Built-in Workload Patterns**
- **Read-Heavy** (80% SELECT, 15% UPDATE, 5% INSERT): Typical reporting workload
- **Write-Heavy** (20% SELECT, 40% UPDATE, 30% INSERT, 10% DELETE): OLTP workload
- **Balanced** (40% SELECT, 30% UPDATE, 20% INSERT, 10% DELETE): Mixed workload
- **CDC-Focused** (10% SELECT, 30% UPDATE, 40% INSERT, 20% DELETE): Maximum change generation

**FR-2: CRUD Operation Generation**
- **SELECT**: Random rows based on primary key or secondary indexes
- **INSERT**: New rows with realistic data (reuse data generators from v0.5.0)
- **UPDATE**: Random columns on random rows
- **DELETE**: Random rows (respecting foreign key constraints)

**FR-3: Concurrent Execution**
- Configurable connection count (1-100 connections)
- Each connection executes operations independently
- Shared transaction rate across all connections
- Connection pooling for efficiency

**FR-4: Rate Control**
- Target transactions per second (TPS): 10, 100, 1000, 10000
- Rate limiting to maintain target TPS
- Report actual TPS vs target TPS

**FR-5: Duration Control**
- Run for specified duration (seconds, minutes, hours)
- Run for specified transaction count
- Run indefinitely until manually stopped
- Graceful shutdown on interrupt (Ctrl+C)

**FR-6: Custom SQL Scripts**
- Execute user-provided SQL files
- Support parameterized queries
- Loop execution with configurable iterations
- Mix custom SQL with built-in patterns

**FR-7: Workload Statistics**
- Total transactions executed
- Success count and error count
- Transactions per second (actual)
- Latency percentiles (p50, p95, p99)
- Error details and frequency

**FR-8: Configuration Format**
```toml
name = "ecommerce_workload"
pattern = "ecommerce"  # oltp, ecommerce, olap, reporting, time_series, social_media, iot, read_heavy, write_heavy, balanced
tables = ["users", "orders"]
connections = 10
target_tps = 100
duration_seconds = 300

[custom_operations]
select_weight = 0.5
insert_weight = 0.25
update_weight = 0.2
delete_weight = 0.05

[[custom_queries]]
name = "update_last_login"
sql = "UPDATE users SET last_login = NOW() WHERE id = $1"
weight = 0.1
```

### Non-Functional Requirements

**NFR-1: Performance**
- Achieve target TPS within ±10%
- Connection overhead <10ms per operation
- Handle 100 concurrent connections without degradation

**NFR-2: Reliability**
- Handle database errors gracefully (retries with exponential backoff)
- Continue running if individual transactions fail
- Log errors without crashing workload

**NFR-3: Accuracy**
- Generated operations respect data constraints
- Foreign key updates reference valid IDs
- DELETE operations handle dependencies correctly

## Implementation Details

### Dependencies

```toml
[dependencies]
sqlx = { version = "0.7", features = ["postgres", "mysql", "mssql", "runtime-tokio-rustls"] }
tokio = { version = "1.36", features = ["full"] }
rand = "0.8"
rand_chacha = "0.3"
hdrhistogram = "7.5"           # Latency percentiles
governor = "0.6"               # Rate limiting
```

### Workload Engine

```rust
pub struct WorkloadEngine {
    config: WorkloadConfig,
    db_pool: DatabasePool,
    rng: ChaCha8Rng,
    rate_limiter: RateLimiter,
    stats: Arc<RwLock<WorkloadStats>>,
}

impl WorkloadEngine {
    pub async fn run(&mut self) -> Result<WorkloadStats> {
        let start_time = Instant::now();
        let mut tasks = Vec::new();

        // Spawn worker tasks (one per connection)
        for worker_id in 0..self.config.connections {
            let worker = self.spawn_worker(worker_id);
            tasks.push(tokio::spawn(worker));
        }

        // Wait for duration or transaction count
        match self.config.stop_condition {
            StopCondition::Duration(duration) => {
                tokio::time::sleep(duration).await;
            }
            StopCondition::TransactionCount(count) => {
                while self.stats.read().total_transactions < count {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            StopCondition::Manual => {
                // Wait for Ctrl+C
                tokio::signal::ctrl_c().await?;
            }
        }

        // Graceful shutdown
        self.shutdown_workers().await;

        Ok(self.stats.read().clone())
    }

    async fn spawn_worker(&self, worker_id: usize) -> Result<()> {
        let mut connection = self.db_pool.acquire().await?;

        loop {
            // Check if should stop
            if self.should_stop() {
                break;
            }

            // Wait for rate limiter
            self.rate_limiter.until_ready().await;

            // Execute operation
            let start = Instant::now();
            let result = self.execute_operation(&mut connection).await;
            let duration = start.elapsed();

            // Record stats
            match result {
                Ok(_) => self.stats.write().record_success(duration),
                Err(e) => self.stats.write().record_error(e),
            }
        }

        Ok(())
    }

    async fn execute_operation(&mut self, conn: &mut Connection) -> Result<()> {
        // Select operation based on pattern
        let operation = self.select_operation();

        match operation {
            Operation::Select => self.execute_select(conn).await,
            Operation::Insert => self.execute_insert(conn).await,
            Operation::Update => self.execute_update(conn).await,
            Operation::Delete => self.execute_delete(conn).await,
            Operation::Custom(sql) => self.execute_custom(conn, sql).await,
        }
    }

    fn select_operation(&mut self) -> Operation {
        let roll: f64 = self.rng.gen();
        let pattern = &self.config.pattern;

        // Cumulative distribution based on pattern
        let mut cumulative = 0.0;

        cumulative += pattern.select_weight;
        if roll < cumulative {
            return Operation::Select;
        }

        cumulative += pattern.insert_weight;
        if roll < cumulative {
            return Operation::Insert;
        }

        cumulative += pattern.update_weight;
        if roll < cumulative {
            return Operation::Update;
        }

        Operation::Delete
    }
}
```

### Operation Generators

```rust
impl WorkloadEngine {
    async fn execute_select(&mut self, conn: &mut Connection) -> Result<()> {
        let table = self.select_random_table();

        // Generate random ID to fetch
        let id = self.rng.gen_range(1..self.table_row_counts[table]);

        let query = format!("SELECT * FROM {} WHERE id = $1", table);
        sqlx::query(&query)
            .bind(id)
            .fetch_optional(conn)
            .await?;

        Ok(())
    }

    async fn execute_insert(&mut self, conn: &mut Connection) -> Result<()> {
        let table = self.select_random_table();
        let schema = &self.schemas[table];

        // Generate row data
        let mut columns = Vec::new();
        let mut values = Vec::new();

        for column in &schema.columns {
            if column.auto_increment {
                continue; // Skip auto-increment columns
            }

            columns.push(&column.name);

            let value = match &column.generator {
                Some(gen) => gen.generate(&mut self.rng),
                None => self.generate_default_value(&column.data_type),
            };

            values.push(value);
        }

        let placeholders: Vec<String> = (1..=values.len())
            .map(|i| format!("${}", i))
            .collect();

        let query = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.join(", "),
            placeholders.join(", ")
        );

        sqlx::query(&query)
            .bind_many(values)
            .execute(conn)
            .await?;

        Ok(())
    }

    async fn execute_update(&mut self, conn: &mut Connection) -> Result<()> {
        let table = self.select_random_table();
        let schema = &self.schemas[table];

        // Select random ID to update
        let id = self.rng.gen_range(1..self.table_row_counts[table]);

        // Select random columns to update (1-3 columns)
        let update_count = self.rng.gen_range(1..=3.min(schema.columns.len()));
        let columns_to_update: Vec<&Column> = schema.columns
            .iter()
            .filter(|c| !c.auto_increment && !c.is_primary_key)
            .collect::<Vec<_>>()
            .choose_multiple(&mut self.rng, update_count)
            .copied()
            .collect();

        let set_clauses: Vec<String> = columns_to_update
            .iter()
            .enumerate()
            .map(|(i, col)| format!("{} = ${}", col.name, i + 1))
            .collect();

        let query = format!(
            "UPDATE {} SET {} WHERE id = ${}",
            table,
            set_clauses.join(", "),
            columns_to_update.len() + 1
        );

        let mut query_builder = sqlx::query(&query);

        for column in &columns_to_update {
            let value = self.generate_default_value(&column.data_type);
            query_builder = query_builder.bind(value);
        }

        query_builder.bind(id).execute(conn).await?;

        Ok(())
    }

    async fn execute_delete(&mut self, conn: &mut Connection) -> Result<()> {
        let table = self.select_random_table();

        // Select random ID to delete
        let id = self.rng.gen_range(1..self.table_row_counts[table]);

        let query = format!("DELETE FROM {} WHERE id = $1", table);
        sqlx::query(&query)
            .bind(id)
            .execute(conn)
            .await?;

        Ok(())
    }
}
```

### Workload Statistics

```rust
use hdrhistogram::Histogram;

pub struct WorkloadStats {
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub latency_histogram: Histogram<u64>,
    pub errors: HashMap<String, u64>,
    pub start_time: Instant,
}

impl WorkloadStats {
    pub fn record_success(&mut self, duration: Duration) {
        self.total_transactions += 1;
        self.successful_transactions += 1;
        self.latency_histogram.record(duration.as_micros() as u64).ok();
    }

    pub fn record_error(&mut self, error: Error) {
        self.total_transactions += 1;
        self.failed_transactions += 1;

        let error_key = format!("{:?}", error);
        *self.errors.entry(error_key).or_insert(0) += 1;
    }

    pub fn report(&self) -> String {
        let elapsed = self.start_time.elapsed();
        let actual_tps = self.total_transactions as f64 / elapsed.as_secs_f64();

        format!(
            "Workload Statistics:\n\
             Total Transactions: {}\n\
             Successful: {} ({:.1}%)\n\
             Failed: {} ({:.1}%)\n\
             Duration: {:.1}s\n\
             Actual TPS: {:.1}\n\
             Latency p50: {:.2}ms\n\
             Latency p95: {:.2}ms\n\
             Latency p99: {:.2}ms\n",
            self.total_transactions,
            self.successful_transactions,
            (self.successful_transactions as f64 / self.total_transactions as f64) * 100.0,
            self.failed_transactions,
            (self.failed_transactions as f64 / self.total_transactions as f64) * 100.0,
            elapsed.as_secs_f64(),
            actual_tps,
            self.latency_histogram.value_at_quantile(0.50) as f64 / 1000.0,
            self.latency_histogram.value_at_quantile(0.95) as f64 / 1000.0,
            self.latency_histogram.value_at_quantile(0.99) as f64 / 1000.0,
        )
    }
}
```

## CLI Interface Design

### Commands

```bash
# Run built-in workload pattern
dbarena workload --container <name> --pattern <pattern> --duration <seconds>

# Run with specific TPS and connections
dbarena workload --container <name> --pattern balanced \
    --connections 10 --tps 100 --duration 300

# Run workload from configuration
dbarena workload --config <config-file> --container <name>
```

### Example Usage

```bash
# Run balanced workload for 5 minutes
$ dbarena workload --container dbarena-postgres-16-a3f9 \
    --pattern balanced --connections 10 --tps 100 --duration 300

Starting workload: balanced
  Container: dbarena-postgres-16-a3f9
  Connections: 10
  Target TPS: 100
  Duration: 300s

Running... [=================] 100% (300s / 300s)
Actual TPS: 98.5 | Latency p50: 12.5ms | Success: 29550/29550 (100%)

Workload complete!

Workload Statistics:
Total Transactions: 29550
Successful: 29550 (100.0%)
Failed: 0 (0.0%)
Duration: 300.0s
Actual TPS: 98.5
Latency p50: 12.50ms
Latency p95: 28.30ms
Latency p99: 45.20ms

# Run write-heavy workload
$ dbarena workload --container dbarena-mysql-8-b7e2 \
    --pattern write_heavy --connections 5 --tps 50 --duration 600

Starting workload: write_heavy
  Container: dbarena-mysql-8-b7e2
  Connections: 5
  Target TPS: 50
  Duration: 600s

Running... [=================] 50% (300s / 600s)
Actual TPS: 49.2 | Latency p50: 15.1ms | Success: 14760/14760 (100%)
```

## Testing Strategy

### Integration Tests
- `test_read_heavy_workload()`: Run read-heavy pattern, verify operations
- `test_write_heavy_workload()`: Run write-heavy pattern, verify changes in database
- `test_concurrent_connections()`: Run with 50 connections, verify no conflicts
- `test_rate_limiting()`: Set TPS limit, verify actual TPS within bounds
- `test_custom_sql()`: Execute custom script, verify results
- `test_workload_duration()`: Run for 60s, verify stops correctly
- `test_error_handling()`: Introduce errors, verify workload continues

### Performance Tests
- `test_tps_achievement()`: Verify can achieve 1000 TPS
- `test_latency_under_load()`: Measure latency with high concurrency
- `test_resource_overhead()`: Measure workload generator resource usage

### Manual Testing
1. **Visual Monitoring**: Run workload while TUI shows metrics
2. **Change Event Monitoring (optional)**: If CDC is configured externally, verify change capture
3. **Long Duration**: Run for 1 hour, verify stability
4. **Database Comparison**: Run identical workload on all three databases

## Documentation Requirements
- **Workload Patterns Reference**: Description of each built-in pattern
- **Configuration Examples**: Sample workload configurations
- **Performance Tuning**: Optimizing workload for target TPS
- **Custom SQL Guide**: Writing effective custom workload scripts

## Future Enhancements
- Think time between transactions (realistic user behavior)
- Transaction support (multi-statement transactions)
- Dependent operations (read then update same row)
- Workload recording and replay
- Adaptive rate adjustment based on latency
- Distributed workload (multiple dbarena instances)
