# Database Metrics Collection

## Feature Overview

Collects database-specific performance metrics including query execution statistics, connection pool status, transaction rates, and replication lag. Provides deep visibility into database behavior beyond basic resource monitoring, enabling performance analysis and CDC-specific monitoring.

**Status:** Implemented. CLI command name is `dbarena` (legacy examples may still show `simdb`). See `specs/IMPLEMENTATION_TRUTH.md`.

## Problem Statement

Resource metrics (CPU, memory, disk) don't tell the full story of database performance:
- High CPU usage could indicate many queries or slow queries - which is it?
- Memory usage doesn't reveal connection pool exhaustion
- Network traffic doesn't show replication lag
- Resource metrics don't capture transaction throughput

Database administrators and developers need database-specific metrics to understand performance characteristics, diagnose issues, and compare database efficiency.

## User Stories

**As a CDC developer**, I want to:
- Monitor replication lag to ensure changes are captured in real-time
- Track change event rates to understand throughput
- See active replication connections and their status
- Identify when replication slots are falling behind

**As a performance engineer**, I want to:
- Track queries per second to measure throughput
- Monitor connection pool utilization to identify bottlenecks
- See transaction commit/rollback rates
- Identify slow queries impacting performance

## Technical Requirements

### Functional Requirements

**FR-1: PostgreSQL Metrics**
- Active connections (total, by state: active, idle, idle in transaction)
- Queries per second (SELECT, INSERT, UPDATE, DELETE)
- Transaction rate (commits/sec, rollbacks/sec)
- Replication lag (for logical replication slots)
- Table sizes and row counts
- Index usage statistics
- Cache hit ratio

**FR-2: MySQL Metrics**
- Active connections and max connections
- Queries per second (COM_SELECT, COM_INSERT, COM_UPDATE, COM_DELETE)
- Transaction rate (commits, rollbacks)
- Binlog position and replication delay
- InnoDB buffer pool hit ratio
- Table lock wait time
- Slow query count

**FR-3: SQL Server Metrics**
- Active connections
- Batch requests per second
- Transaction rate (transactions/sec)
- Page life expectancy (buffer pool efficiency)
- Lock waits per second
- CDC/CT capture latency
- Database size and log size

**FR-4: Collection Strategy**
- Poll metrics every 1-5 seconds (configurable)
- Use database-native system views and functions
- Minimize impact on database performance (<1% overhead)
- Handle connection failures gracefully

**FR-5: Metrics Storage**
- Store in same time-series storage as resource metrics (from v0.3.0)
- Retain for configurable duration (default: 1 hour)
- Support querying by metric type and time range

### Non-Functional Requirements

**NFR-1: Performance**
- Metrics collection queries complete in <100ms
- Collection overhead <1% of database CPU usage
- Support monitoring up to 10 containers simultaneously

**NFR-2: Reliability**
- Retry failed metric queries (up to 3 attempts)
- Continue monitoring if individual metrics fail
- Log errors without crashing collector

## Implementation Details

### PostgreSQL Metrics Queries

```sql
-- Active connections by state
SELECT state, COUNT(*) as count
FROM pg_stat_activity
WHERE datname = current_database()
GROUP BY state;

-- Queries per second (derived from pg_stat_database)
SELECT
    numbackends as connections,
    xact_commit as commits,
    xact_rollback as rollbacks,
    tup_returned as rows_returned,
    tup_fetched as rows_fetched,
    tup_inserted as rows_inserted,
    tup_updated as rows_updated,
    tup_deleted as rows_deleted
FROM pg_stat_database
WHERE datname = current_database();

-- Replication lag
SELECT
    slot_name,
    pg_wal_lsn_diff(pg_current_wal_lsn(), confirmed_flush_lsn) as lag_bytes
FROM pg_replication_slots
WHERE slot_type = 'logical';

-- Cache hit ratio
SELECT
    sum(heap_blks_hit) / nullif(sum(heap_blks_hit + heap_blks_read), 0) * 100 as cache_hit_ratio
FROM pg_statio_user_tables;
```

### MySQL Metrics Queries

```sql
-- Connection stats
SHOW STATUS LIKE 'Threads_connected';
SHOW STATUS LIKE 'Max_used_connections';
SHOW VARIABLES LIKE 'max_connections';

-- Query stats (counters, calculate rate over time)
SHOW GLOBAL STATUS LIKE 'Com_select';
SHOW GLOBAL STATUS LIKE 'Com_insert';
SHOW GLOBAL STATUS LIKE 'Com_update';
SHOW GLOBAL STATUS LIKE 'Com_delete';

-- InnoDB buffer pool
SHOW GLOBAL STATUS LIKE 'Innodb_buffer_pool_read_requests';
SHOW GLOBAL STATUS LIKE 'Innodb_buffer_pool_reads';

-- Replication status
SHOW SLAVE STATUS;  -- For replication lag
SHOW MASTER STATUS; -- For binlog position
```

### SQL Server Metrics Queries

```sql
-- Active connections
SELECT COUNT(*) as active_connections
FROM sys.dm_exec_sessions
WHERE is_user_process = 1;

-- Batch requests per second
SELECT cntr_value
FROM sys.dm_os_performance_counters
WHERE counter_name = 'Batch Requests/sec';

-- Transactions per second
SELECT cntr_value
FROM sys.dm_os_performance_counters
WHERE counter_name = 'Transactions/sec'
AND instance_name = '_Total';

-- Page life expectancy
SELECT cntr_value as page_life_expectancy_seconds
FROM sys.dm_os_performance_counters
WHERE counter_name = 'Page life expectancy'
AND object_name LIKE '%Buffer Manager%';
```

## Testing Strategy

### Integration Tests
- `test_postgres_metrics_collection()`: Verify all PostgreSQL metrics collected
- `test_mysql_metrics_collection()`: Verify all MySQL metrics collected
- `test_sqlserver_metrics_collection()`: Verify all SQL Server metrics collected
- `test_replication_lag_detection()`: Create replication delay, verify detection
- `test_connection_pool_metrics()`: Create connections, verify count
- `test_query_rate_calculation()`: Execute queries, verify QPS calculation

## Documentation Requirements
- **Metrics Reference**: Complete list of collected metrics per database type
- **Query Details**: SQL queries used for each metric
- **Performance Impact**: Expected overhead of metrics collection

## Future Enhancements
- Custom metrics via user-defined queries
- Alerting based on metric thresholds
- Metrics aggregation and rollups
- Detailed query performance tracking (query plans, execution times)
