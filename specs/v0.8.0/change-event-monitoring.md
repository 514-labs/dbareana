# Change Event Monitoring

## Feature Overview

Real-time monitoring system for CDC change events, providing visibility into captured database changes across PostgreSQL, MySQL, and SQL Server. Enables inspection, filtering, and analysis of change streams to validate CDC behavior and diagnose issues.

## Problem Statement

CDC systems capture database changes, but validating correct behavior requires:
- Seeing actual change events, not just database logs
- Measuring change capture rate and lag
- Identifying missing or duplicate events
- Debugging CDC connector issues
- Comparing change formats across databases

Without change event monitoring, CDC developers must:
- Connect external tools to inspect changes
- Write custom scripts to consume change streams
- Manually correlate workload with captured events
- Debug blind without visibility into change stream

## User Stories

**As a CDC developer**, I want to:
- See PostgreSQL logical replication changes as they're captured
- Verify all INSERT/UPDATE/DELETE operations are captured
- Measure change event rate during load testing
- Filter changes by table to focus on specific entities
- Debug CDC lag by seeing which changes are delayed

**As a QA engineer**, I want to:
- Verify CDC captures exactly N changes after N database operations
- Inspect change event format to validate connector output
- Ensure no duplicate or missing events during testing
- Compare change capture behavior across databases

## Technical Requirements

### Functional Requirements

**FR-1: PostgreSQL Change Stream Consumption**
- Connect to logical replication slot
- Decode change events using test_decoding or pgoutput
- Parse change events into structured format
- Handle LSN (Log Sequence Number) tracking

**FR-2: MySQL Binlog Consumption**
- Connect to MySQL binlog stream
- Parse binlog events (WRITE_ROWS, UPDATE_ROWS, DELETE_ROWS)
- Handle binlog position tracking
- Support ROW format binlog events

**FR-3: SQL Server CDC Consumption**
- Query CDC change tables (cdc.fn_cdc_get_all_changes)
- Parse CDC change records
- Handle LSN tracking for SQL Server
- Support both CDC and Change Tracking

**FR-4: Change Event Display**
- Show event metadata (timestamp, table, operation type)
- Display before/after values for UPDATE operations
- Format output as JSON or table view
- Color-code by operation type (INSERT=green, UPDATE=yellow, DELETE=red)

**FR-5: Event Rate Metrics**
- Track events per second by operation type
- Display cumulative event counts
- Calculate average event rate over time windows
- Show event rate graph in TUI

**FR-6: Filtering and Search**
- Filter by table name
- Filter by operation type (INSERT/UPDATE/DELETE)
- Filter by time range
- Search for specific values in change events

**FR-7: Replication Lag Monitoring**
- Calculate lag between database operation and event capture
- Display lag in bytes (PostgreSQL) or seconds (MySQL, SQL Server)
- Alert when lag exceeds threshold
- Show lag trend over time

### Non-Functional Requirements

**NFR-1: Performance**
- Monitor up to 10,000 events per second without dropping events
- Event display latency <100ms
- Minimal overhead on database performance

**NFR-2: Reliability**
- Handle connection interruptions (reconnect automatically)
- No event loss during monitoring
- Graceful handling of malformed events

## Implementation Details

### PostgreSQL Change Stream Monitor

```rust
use postgres::Client;

pub struct PostgresChangeMonitor {
    client: Client,
    slot_name: String,
    lsn: Option<String>,
}

impl PostgresChangeMonitor {
    pub async fn start_monitoring(&mut self) -> Result<()> {
        let stream = self.client.copy_out(
            &format!(
                "START_REPLICATION SLOT {} LOGICAL 0/0",
                self.slot_name
            )
        )?;

        for message in stream {
            match self.parse_change_event(&message) {
                Ok(event) => {
                    self.emit_event(event).await?;
                }
                Err(e) => {
                    tracing::warn!("Failed to parse event: {}", e);
                }
            }
        }

        Ok(())
    }

    fn parse_change_event(&self, data: &[u8]) -> Result<ChangeEvent> {
        // Parse test_decoding output format:
        // table public.users: INSERT: id[integer]:123 name[text]:'John' email[text]:'john@example.com'
        let text = String::from_utf8_lossy(data);

        if text.starts_with("table") {
            let parts: Vec<&str> = text.split(':').collect();
            let table = parts[0].trim_start_matches("table ").trim();
            let operation = parts[1].trim();

            let fields = self.parse_fields(&parts[2..].join(":"));

            Ok(ChangeEvent {
                timestamp: Utc::now(),
                table: table.to_string(),
                operation: operation.to_string(),
                before: None,
                after: Some(fields),
                lsn: self.lsn.clone(),
            })
        } else {
            Err(anyhow!("Unknown message format"))
        }
    }

    fn parse_fields(&self, field_str: &str) -> HashMap<String, String> {
        let mut fields = HashMap::new();

        // Parse: id[integer]:123 name[text]:'John'
        for field in field_str.split_whitespace() {
            if let Some((key_type, value)) = field.split_once(':') {
                if let Some((key, _type)) = key_type.split_once('[') {
                    fields.insert(key.to_string(), value.trim_matches('\'').to_string());
                }
            }
        }

        fields
    }

    async fn emit_event(&self, event: ChangeEvent) -> Result<()> {
        // Send to event channel for display/processing
        EVENT_CHANNEL.send(event).await?;
        Ok(())
    }
}
```

### MySQL Binlog Monitor

```rust
use mysql_cdc::{BinlogStream, BinlogEvent};

pub struct MySQLChangeMonitor {
    stream: BinlogStream,
    position: BinlogPosition,
}

impl MySQLChangeMonitor {
    pub async fn start_monitoring(&mut self) -> Result<()> {
        while let Some(event) = self.stream.next().await? {
            match event {
                BinlogEvent::WriteRows(e) => {
                    for row in e.rows() {
                        let change_event = ChangeEvent {
                            timestamp: e.timestamp(),
                            table: format!("{}.{}", e.schema_name(), e.table_name()),
                            operation: "INSERT".to_string(),
                            before: None,
                            after: Some(self.row_to_map(row)),
                            lsn: Some(format!("{}:{}", e.log_file(), e.log_pos())),
                        };
                        self.emit_event(change_event).await?;
                    }
                }

                BinlogEvent::UpdateRows(e) => {
                    for (before, after) in e.rows() {
                        let change_event = ChangeEvent {
                            timestamp: e.timestamp(),
                            table: format!("{}.{}", e.schema_name(), e.table_name()),
                            operation: "UPDATE".to_string(),
                            before: Some(self.row_to_map(before)),
                            after: Some(self.row_to_map(after)),
                            lsn: Some(format!("{}:{}", e.log_file(), e.log_pos())),
                        };
                        self.emit_event(change_event).await?;
                    }
                }

                BinlogEvent::DeleteRows(e) => {
                    for row in e.rows() {
                        let change_event = ChangeEvent {
                            timestamp: e.timestamp(),
                            table: format!("{}.{}", e.schema_name(), e.table_name()),
                            operation: "DELETE".to_string(),
                            before: Some(self.row_to_map(row)),
                            after: None,
                            lsn: Some(format!("{}:{}", e.log_file(), e.log_pos())),
                        };
                        self.emit_event(change_event).await?;
                    }
                }

                _ => {} // Ignore other events
            }
        }

        Ok(())
    }

    fn row_to_map(&self, row: &Row) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (i, value) in row.values().enumerate() {
            map.insert(
                format!("col_{}", i),
                format!("{:?}", value)
            );
        }
        map
    }
}
```

### SQL Server CDC Monitor

```rust
pub struct SQLServerChangeMonitor {
    connection: MssqlConnection,
    last_lsn: Option<Vec<u8>>,
    tables: Vec<String>,
}

impl SQLServerChangeMonitor {
    pub async fn start_monitoring(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            for table in &self.tables {
                let events = self.fetch_changes(table).await?;
                for event in events {
                    self.emit_event(event).await?;
                }
            }
        }
    }

    async fn fetch_changes(&mut self, table: &str) -> Result<Vec<ChangeEvent>> {
        let parts: Vec<&str> = table.split('.').collect();
        let (schema, table_name) = (parts[0], parts[1]);

        let capture_instance = format!("{}_{}", schema, table_name);

        let query = format!(
            "SELECT * FROM cdc.fn_cdc_get_all_changes_{}(@from_lsn, @to_lsn, 'all')",
            capture_instance
        );

        let from_lsn = self.last_lsn.clone()
            .unwrap_or_else(|| self.get_min_lsn(&capture_instance));

        let to_lsn = self.get_max_lsn();

        let rows = sqlx::query(&query)
            .bind(&from_lsn)
            .bind(&to_lsn)
            .fetch_all(&self.connection)
            .await?;

        let mut events = Vec::new();

        for row in rows {
            let operation = row.get::<i32, _>("__$operation");
            let operation_str = match operation {
                1 => "DELETE",
                2 => "INSERT",
                3 => "UPDATE_BEFORE",
                4 => "UPDATE_AFTER",
                _ => "UNKNOWN",
            };

            // For UPDATE operations, we get both BEFORE and AFTER rows
            // Combine them into a single UPDATE event
            if operation_str == "UPDATE_BEFORE" || operation_str == "UPDATE_AFTER" {
                // Handle UPDATE logic...
            }

            let event = ChangeEvent {
                timestamp: row.get("__$start_lsn_time"),
                table: table.to_string(),
                operation: operation_str.to_string(),
                before: None,
                after: Some(self.row_to_map(&row)),
                lsn: Some(hex::encode(row.get::<Vec<u8>, _>("__$start_lsn"))),
            };

            events.push(event);
        }

        if !events.is_empty() {
            self.last_lsn = Some(to_lsn);
        }

        Ok(events)
    }
}
```

### Change Event Model

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ChangeEvent {
    pub timestamp: DateTime<Utc>,
    pub table: String,
    pub operation: String,  // INSERT, UPDATE, DELETE
    pub before: Option<HashMap<String, String>>,
    pub after: Option<HashMap<String, String>>,
    pub lsn: Option<String>,  // LSN or binlog position
}

impl ChangeEvent {
    pub fn display_terminal(&self) -> String {
        let operation_colored = match self.operation.as_str() {
            "INSERT" => style(&self.operation).green(),
            "UPDATE" => style(&self.operation).yellow(),
            "DELETE" => style(&self.operation).red(),
            _ => style(&self.operation).white(),
        };

        let mut output = format!(
            "[{}] {} on {}\n",
            self.timestamp.format("%H:%M:%S"),
            operation_colored,
            style(&self.table).bold()
        );

        if let Some(before) = &self.before {
            output.push_str(&format!("  Before: {:?}\n", before));
        }

        if let Some(after) = &self.after {
            output.push_str(&format!("  After:  {:?}\n", after));
        }

        output
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}
```

### Event Rate Tracker

```rust
pub struct EventRateTracker {
    insert_count: AtomicU64,
    update_count: AtomicU64,
    delete_count: AtomicU64,
    window_start: Instant,
}

impl EventRateTracker {
    pub fn record(&self, event: &ChangeEvent) {
        match event.operation.as_str() {
            "INSERT" => self.insert_count.fetch_add(1, Ordering::Relaxed),
            "UPDATE" => self.update_count.fetch_add(1, Ordering::Relaxed),
            "DELETE" => self.delete_count.fetch_add(1, Ordering::Relaxed),
            _ => 0,
        };
    }

    pub fn get_rates(&self) -> EventRates {
        let elapsed = self.window_start.elapsed().as_secs_f64();

        EventRates {
            inserts_per_sec: self.insert_count.load(Ordering::Relaxed) as f64 / elapsed,
            updates_per_sec: self.update_count.load(Ordering::Relaxed) as f64 / elapsed,
            deletes_per_sec: self.delete_count.load(Ordering::Relaxed) as f64 / elapsed,
        }
    }

    pub fn reset(&mut self) {
        self.insert_count.store(0, Ordering::Relaxed);
        self.update_count.store(0, Ordering::Relaxed);
        self.delete_count.store(0, Ordering::Relaxed);
        self.window_start = Instant::now();
    }
}
```

## CLI Interface Design

### Commands

```bash
# Monitor change events in real-time
simdb cdc monitor --container <name>

# Monitor with filtering
simdb cdc monitor --container <name> --table users --operation INSERT

# Monitor with JSON output
simdb cdc monitor --container <name> --json

# Show event rate statistics
simdb cdc stats --container <name>

# Show replication lag
simdb cdc lag --container <name>
```

### Example Usage

```bash
# Monitor PostgreSQL changes
$ simdb cdc monitor --container simdb-postgres-16-a3f9
Monitoring change events from simdb-postgres-16-a3f9 (replication slot: simdb_slot)
Press Ctrl+C to stop

[14:35:21] INSERT on public.users
  After:  {"id": "123", "email": "test@example.com", "created_at": "2026-01-22 14:35:21"}

[14:35:22] UPDATE on public.orders
  Before: {"id": "456", "status": "pending", "total": "99.99"}
  After:  {"id": "456", "status": "completed", "total": "99.99"}

[14:35:23] DELETE on public.orders
  Before: {"id": "789", "status": "cancelled", "total": "149.99"}

Event Rate: 45.2 events/sec (INS: 20.1, UPD: 18.3, DEL: 6.8) | Lag: 0 bytes

# Monitor with filtering
$ simdb cdc monitor --container simdb-mysql-8-b7e2 --table users --operation INSERT
Monitoring INSERT events on table 'users'

[14:35:25] INSERT on testdb.users
  After: {"id": "124", "email": "alice@example.com"}

[14:35:26] INSERT on testdb.users
  After: {"id": "125", "email": "bob@example.com"}

# Show event rate statistics
$ simdb cdc stats --container simdb-postgres-16-a3f9
Change Event Statistics (last 60 seconds):
  Total Events: 2,750
  INSERT: 1,200 (43.6%)
  UPDATE: 1,100 (40.0%)
  DELETE: 450 (16.4%)
  Average Rate: 45.8 events/sec
  Peak Rate: 87.3 events/sec

# Show replication lag
$ simdb cdc lag --container simdb-mysql-8-b7e2
Replication Lag:
  Current Lag: 2.3 seconds
  Binlog Position: mysql-bin.000003:45678
  Events Behind: ~115
```

## Testing Strategy

### Integration Tests
- `test_postgres_change_monitoring()`: Generate changes, verify all captured
- `test_mysql_binlog_monitoring()`: Monitor binlog events, verify correctness
- `test_sqlserver_cdc_monitoring()`: Poll CDC tables, verify events
- `test_event_rate_tracking()`: Generate 1000 events, verify rate calculation
- `test_event_filtering()`: Filter by table and operation, verify only matching events
- `test_no_event_loss()`: Generate high-rate changes, verify zero loss

### Manual Testing
1. **Visual Verification**: Monitor changes while TUI shows workload
2. **Lag Measurement**: Introduce delay, verify lag detection
3. **Long-Running**: Monitor for 1 hour, verify stability
4. **High Throughput**: Generate 10K events/sec, verify monitoring keeps up
5. **Cross-Database**: Compare change formats across all three databases

## Documentation Requirements
- **Change Event Format Reference**: Structure of events for each database
- **Monitoring Guide**: Best practices for change event monitoring
- **Performance Tuning**: Optimizing for high-throughput monitoring
- **Troubleshooting**: Common issues (connection drops, parse errors)

## Future Enhancements
- Change event persistence (save to file for later analysis)
- Change event replay (re-execute captured changes)
- Diff tool (compare change streams from two databases)
- Event validation (verify referential integrity in changes)
- Custom event handlers (webhooks, alerting)
- Grafana integration for change event dashboards
