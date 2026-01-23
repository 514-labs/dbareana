# CDC Configuration Support

## Feature Overview

Comprehensive CDC configuration system for PostgreSQL, MySQL, and SQL Server. Provides guided setup, validation, and management of Change Data Capture features specific to each database, enabling users to quickly configure databases for CDC testing without manual database administration.

## Problem Statement

Enabling CDC requires database-specific configuration:
- **PostgreSQL**: WAL level, replication slots, logical replication configuration
- **MySQL**: Binlog enabling, format configuration, retention settings
- **SQL Server**: CDC enabling at database level, table-level enablement, agent configuration

Manual CDC configuration is error-prone:
- Missing prerequisites (e.g., wrong WAL level in PostgreSQL)
- Incorrect settings (e.g., STATEMENT binlog format instead of ROW in MySQL)
- Permission issues
- Lack of validation before CDC connector deployment

CDC developers need a reliable way to spin up CDC-enabled databases for testing.

## User Stories

**As a CDC developer**, I want to:
- Enable PostgreSQL logical replication with a single command
- Create a replication slot for my CDC connector
- Verify CDC configuration is correct before starting my connector
- See replication slot status and lag

**As a MySQL CDC developer**, I want to:
- Enable binlog with ROW format automatically
- Verify binlog is capturing changes correctly
- Check binlog position and available binlogs

**As a SQL Server CDC developer**, I want to:
- Enable CDC on specific tables without manual TSQL
- Verify CDC capture process is running
- Check CDC latency and table coverage

## Technical Requirements

### Functional Requirements

**FR-1: PostgreSQL CDC Configuration**
- Set `wal_level = logical` (requires container restart)
- Set `max_replication_slots` and `max_wal_senders`
- Create logical replication slots with pgoutput or test_decoding plugin
- Grant replication permissions to users
- Verify configuration is valid

**FR-2: MySQL CDC Configuration**
- Enable binlog (`log_bin = ON`)
- Set binlog format (`binlog_format = ROW`)
- Set binlog retention (binlog_expire_logs_seconds)
- Set server_id (required for replication)
- Grant REPLICATION SLAVE, REPLICATION CLIENT permissions
- Verify binlog is enabled and accessible

**FR-3: SQL Server CDC Configuration**
- Enable CDC at database level (`EXEC sys.sp_cdc_enable_db`)
- Enable CDC for specific tables (`sys.sp_cdc_enable_table`)
- Verify SQL Server Agent is running (required for CDC)
- Configure CDC capture and cleanup jobs
- Grant db_owner or cdc_admin permissions

**FR-4: Configuration Validation**
- Pre-flight checks before enabling CDC
- Validate prerequisites (permissions, database version)
- Verify configuration after enabling
- Clear error messages if configuration fails

**FR-5: Configuration Persistence**
- Container environment variables for CDC settings
- Configuration survives container restarts
- Document manual steps if full automation not possible

**FR-6: Configuration Templates**
```toml
[cdc]
enabled = true
type = "postgres"  # or mysql, sqlserver

  [cdc.postgres]
  wal_level = "logical"
  max_replication_slots = 10
  max_wal_senders = 10

  [[cdc.postgres.replication_slots]]
  name = "test_slot"
  plugin = "pgoutput"

[cdc]
enabled = true
type = "mysql"

  [cdc.mysql]
  binlog_format = "ROW"
  binlog_expire_logs_seconds = 604800  # 7 days
  server_id = 1

[cdc]
enabled = true
type = "sqlserver"

  [cdc.sqlserver]
  tables = ["dbo.users", "dbo.orders"]
  capture_instance_name_prefix = "cdc_"
```

### Non-Functional Requirements

**NFR-1: Reliability**
- Configuration commands are idempotent (safe to run multiple times)
- Handle partial failures gracefully
- Rollback on critical errors

**NFR-2: Usability**
- Clear documentation of CDC requirements per database
- Helpful error messages with actionable steps
- Validate before applying (dry-run mode)

## Implementation Details

### PostgreSQL CDC Configuration

```rust
pub struct PostgresCdcConfigurator {
    connection: PgConnection,
}

impl PostgresCdcConfigurator {
    pub async fn enable_cdc(&self, config: &PostgresCdcConfig) -> Result<()> {
        // 1. Check current WAL level
        let current_wal_level: String = sqlx::query_scalar(
            "SHOW wal_level"
        ).fetch_one(&self.connection).await?;

        if current_wal_level != "logical" {
            return Err(anyhow!(
                "WAL level is '{}', must be 'logical'. \
                 Add 'wal_level=logical' to postgresql.conf and restart container.",
                current_wal_level
            ));
        }

        // 2. Check replication slots
        let max_replication_slots: i32 = sqlx::query_scalar(
            "SHOW max_replication_slots"
        ).fetch_one(&self.connection).await?;

        if max_replication_slots == 0 {
            return Err(anyhow!(
                "max_replication_slots is 0. \
                 Set max_replication_slots >= 1 in postgresql.conf."
            ));
        }

        // 3. Create replication slot
        for slot in &config.replication_slots {
            self.create_replication_slot(&slot.name, &slot.plugin).await?;
        }

        Ok(())
    }

    async fn create_replication_slot(&self, slot_name: &str, plugin: &str) -> Result<()> {
        // Check if slot already exists
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_replication_slots WHERE slot_name = $1)"
        )
        .bind(slot_name)
        .fetch_one(&self.connection)
        .await?;

        if exists {
            println!("Replication slot '{}' already exists", slot_name);
            return Ok(());
        }

        // Create slot
        sqlx::query(&format!(
            "SELECT pg_create_logical_replication_slot($1, $2)"
        ))
        .bind(slot_name)
        .bind(plugin)
        .execute(&self.connection)
        .await?;

        println!("✓ Created replication slot: {}", slot_name);
        Ok(())
    }

    pub async fn get_replication_slot_status(&self, slot_name: &str) -> Result<ReplicationSlotStatus> {
        let row: (String, String, i64) = sqlx::query_as(
            "SELECT slot_name, slot_type, \
             pg_wal_lsn_diff(pg_current_wal_lsn(), confirmed_flush_lsn) as lag_bytes \
             FROM pg_replication_slots \
             WHERE slot_name = $1"
        )
        .bind(slot_name)
        .fetch_one(&self.connection)
        .await?;

        Ok(ReplicationSlotStatus {
            slot_name: row.0,
            slot_type: row.1,
            lag_bytes: row.2,
        })
    }
}
```

### MySQL CDC Configuration

```rust
pub struct MySQLCdcConfigurator {
    connection: MySqlConnection,
}

impl MySQLCdcConfigurator {
    pub async fn enable_cdc(&self, config: &MySQLCdcConfig) -> Result<()> {
        // 1. Check binlog status
        let binlog_status: (String,) = sqlx::query_as(
            "SHOW VARIABLES LIKE 'log_bin'"
        ).fetch_one(&self.connection).await?;

        if binlog_status.0 != "ON" {
            return Err(anyhow!(
                "Binary logging is not enabled. \
                 Add 'log_bin=mysql-bin' to my.cnf and restart container."
            ));
        }

        // 2. Check binlog format
        let binlog_format: (String,) = sqlx::query_as(
            "SHOW VARIABLES LIKE 'binlog_format'"
        ).fetch_one(&self.connection).await?;

        if binlog_format.0 != "ROW" {
            return Err(anyhow!(
                "Binlog format is '{}', must be 'ROW' for CDC. \
                 Set binlog_format=ROW in my.cnf.",
                binlog_format.0
            ));
        }

        // 3. Verify server_id is set
        let server_id: (i32,) = sqlx::query_as(
            "SHOW VARIABLES LIKE 'server_id'"
        ).fetch_one(&self.connection).await?;

        if server_id.0 == 0 {
            return Err(anyhow!(
                "server_id is 0. Set server_id to a unique non-zero value in my.cnf."
            ));
        }

        println!("✓ MySQL CDC configuration is valid");
        println!("  - Binlog: ON");
        println!("  - Binlog Format: ROW");
        println!("  - Server ID: {}", server_id.0);

        Ok(())
    }

    pub async fn get_binlog_status(&self) -> Result<BinlogStatus> {
        let row: (String, i64, String, String) = sqlx::query_as(
            "SHOW MASTER STATUS"
        ).fetch_one(&self.connection).await?;

        Ok(BinlogStatus {
            file: row.0,
            position: row.1,
            binlog_do_db: row.2,
            binlog_ignore_db: row.3,
        })
    }

    pub async fn list_binlogs(&self) -> Result<Vec<BinlogFile>> {
        let rows: Vec<(String, i64)> = sqlx::query_as(
            "SHOW BINARY LOGS"
        ).fetch_all(&self.connection).await?;

        Ok(rows.into_iter().map(|(name, size)| BinlogFile { name, size }).collect())
    }
}
```

### SQL Server CDC Configuration

```rust
pub struct SQLServerCdcConfigurator {
    connection: MssqlConnection,
}

impl SQLServerCdcConfigurator {
    pub async fn enable_cdc(&self, config: &SQLServerCdcConfig) -> Result<()> {
        // 1. Check if SQL Server Agent is running (required for CDC)
        let agent_status: bool = sqlx::query_scalar(
            "SELECT CASE WHEN EXISTS (
                SELECT 1 FROM sys.dm_server_services
                WHERE servicename LIKE 'SQL Server Agent%' AND status = 4
            ) THEN 1 ELSE 0 END"
        ).fetch_one(&self.connection).await?;

        if !agent_status {
            println!("WARNING: SQL Server Agent is not running. CDC requires SQL Server Agent.");
            println!("In container, run: /opt/mssql/bin/mssql-conf set sqlagent.enabled true");
        }

        // 2. Enable CDC at database level
        let is_cdc_enabled: bool = sqlx::query_scalar(
            "SELECT is_cdc_enabled FROM sys.databases WHERE name = DB_NAME()"
        ).fetch_one(&self.connection).await?;

        if !is_cdc_enabled {
            println!("Enabling CDC at database level...");
            sqlx::query("EXEC sys.sp_cdc_enable_db")
                .execute(&self.connection)
                .await?;
            println!("✓ CDC enabled at database level");
        } else {
            println!("✓ CDC already enabled at database level");
        }

        // 3. Enable CDC for specific tables
        for table in &config.tables {
            self.enable_cdc_for_table(table).await?;
        }

        Ok(())
    }

    async fn enable_cdc_for_table(&self, table: &str) -> Result<()> {
        // Parse schema and table name
        let parts: Vec<&str> = table.split('.').collect();
        let (schema, table_name) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            ("dbo", parts[0])
        };

        // Check if CDC already enabled for this table
        let is_enabled: bool = sqlx::query_scalar(
            "SELECT CASE WHEN EXISTS (
                SELECT 1 FROM cdc.change_tables ct
                JOIN sys.tables t ON ct.source_object_id = t.object_id
                JOIN sys.schemas s ON t.schema_id = s.schema_id
                WHERE s.name = @p1 AND t.name = @p2
            ) THEN 1 ELSE 0 END"
        )
        .bind(schema)
        .bind(table_name)
        .fetch_one(&self.connection)
        .await?;

        if is_enabled {
            println!("✓ CDC already enabled for {}.{}", schema, table_name);
            return Ok(());
        }

        // Enable CDC for table
        println!("Enabling CDC for {}.{}...", schema, table_name);
        sqlx::query(&format!(
            "EXEC sys.sp_cdc_enable_table \
             @source_schema = N'{}', \
             @source_name = N'{}', \
             @role_name = NULL, \
             @supports_net_changes = 1",
            schema, table_name
        ))
        .execute(&self.connection)
        .await?;

        println!("✓ CDC enabled for {}.{}", schema, table_name);
        Ok(())
    }

    pub async fn get_cdc_status(&self) -> Result<CdcStatus> {
        let is_enabled: bool = sqlx::query_scalar(
            "SELECT is_cdc_enabled FROM sys.databases WHERE name = DB_NAME()"
        ).fetch_one(&self.connection).await?;

        let enabled_tables: Vec<String> = sqlx::query_scalar(
            "SELECT s.name + '.' + t.name as table_name \
             FROM cdc.change_tables ct \
             JOIN sys.tables t ON ct.source_object_id = t.object_id \
             JOIN sys.schemas s ON t.schema_id = s.schema_id"
        ).fetch_all(&self.connection).await?;

        Ok(CdcStatus {
            database_cdc_enabled: is_enabled,
            enabled_tables,
        })
    }
}
```

## CLI Interface Design

### Commands

```bash
# Enable CDC with guided setup
simdb cdc enable --container <name>

# Enable CDC from configuration
simdb cdc enable --container <name> --config <config-file>

# Validate CDC configuration without enabling
simdb cdc validate --container <name>

# Show CDC status
simdb cdc status --container <name>

# Create PostgreSQL replication slot
simdb cdc create-slot --container <name> --slot-name <name> [--plugin pgoutput]

# Show PostgreSQL replication slot status
simdb cdc slot-status --container <name> --slot-name <name>

# Show MySQL binlog status
simdb cdc binlog-status --container <name>
```

### Example Usage

```bash
# Enable PostgreSQL CDC
$ simdb cdc enable --container simdb-postgres-16-a3f9
Validating CDC prerequisites...
  ✓ WAL level: logical
  ✓ max_replication_slots: 10
  ✓ max_wal_senders: 10
Creating replication slot: simdb_slot
  ✓ Created replication slot: simdb_slot (plugin: pgoutput)
CDC enabled successfully!

# Show replication slot status
$ simdb cdc slot-status --container simdb-postgres-16-a3f9 --slot-name simdb_slot
Replication Slot: simdb_slot
  Type: logical
  Plugin: pgoutput
  Active: false
  Lag: 0 bytes

# Enable MySQL CDC
$ simdb cdc enable --container simdb-mysql-8-b7e2
Validating CDC prerequisites...
  ✓ Binlog: ON
  ✓ Binlog Format: ROW
  ✓ Server ID: 1
CDC configuration is valid!

# Show MySQL binlog status
$ simdb cdc binlog-status --container simdb-mysql-8-b7e2
Binlog Status:
  Current File: mysql-bin.000003
  Position: 154
  Available Binlogs:
    - mysql-bin.000001 (512 MB)
    - mysql-bin.000002 (512 MB)
    - mysql-bin.000003 (128 MB)

# Enable SQL Server CDC
$ simdb cdc enable --container simdb-sqlserver-22-c9d4 --config cdc-config.toml
Validating CDC prerequisites...
  ⚠ SQL Server Agent is not running (CDC jobs may not execute)
Enabling CDC at database level...
  ✓ CDC enabled at database level
Enabling CDC for tables...
  ✓ CDC enabled for dbo.users
  ✓ CDC enabled for dbo.orders
CDC enabled successfully!

# Show SQL Server CDC status
$ simdb cdc status --container simdb-sqlserver-22-c9d4
SQL Server CDC Status:
  Database CDC Enabled: Yes
  Enabled Tables:
    - dbo.users
    - dbo.orders
  CDC Capture Job: Running
  CDC Cleanup Job: Running
```

## Testing Strategy

### Integration Tests
- `test_postgres_cdc_enable()`: Enable CDC, verify replication slot created
- `test_mysql_cdc_validate()`: Validate MySQL binlog configuration
- `test_sqlserver_cdc_enable()`: Enable CDC, verify tables enabled
- `test_postgres_slot_status()`: Create slot, check status
- `test_mysql_binlog_status()`: Check binlog files and position
- `test_cdc_configuration_persistence()`: Restart container, verify CDC still enabled

### Manual Testing
1. **PostgreSQL**: Create slot, use pg_recvlogical to consume changes
2. **MySQL**: Enable binlog, use mysqlbinlog to read events
3. **SQL Server**: Enable CDC, query cdc.fn_cdc_get_all_changes
4. **Integration**: Run workload, verify changes captured by CDC
5. **TUI Integration**: Enable CDC, verify status displayed in TUI

## Documentation Requirements
- **CDC Setup Guide**: Step-by-step CDC configuration for each database
- **Prerequisites**: Required database versions and settings
- **Troubleshooting**: Common CDC configuration issues and solutions
- **Connector Integration**: Connecting Debezium/Kafka Connect to simDB

## Future Enhancements
- Automatic container restart after configuration changes
- CDC connector deployment (Debezium, custom connectors)
- Change stream visualization in TUI
- CDC performance tuning recommendations
- Multi-database CDC synchronization (e.g., Postgres → Postgres replication)
