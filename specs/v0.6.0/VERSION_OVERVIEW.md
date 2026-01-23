# Version 0.6.0 - Workload Generation

## Release Summary

This release introduces comprehensive workload generation capabilities, enabling users to simulate realistic database transactions and concurrent operations. The workload system generates CRUD operations against seeded data, supports custom SQL scripts, and simulates concurrent connections to test database behavior under load.

## Key Features

- **Built-in Workload Patterns**: Read-heavy, write-heavy, balanced OLTP patterns
- **Concurrent Connection Simulation**: Configurable number of concurrent database connections
- **CRUD Operation Generator**: Automatic INSERT, UPDATE, DELETE operations on seeded data
- **Custom SQL Script Execution**: User-defined SQL for specific test scenarios
- **Transaction Rate Control**: Configurable transactions per second (TPS)
- **Workload Duration Control**: Run for specified time or transaction count

## Value Proposition

This release enables realistic CDC testing by generating database activity. Users can now:
- Generate continuous stream of changes for CDC systems to capture
- Test CDC performance under various transaction rates
- Simulate realistic OLTP workloads without application code
- Measure database performance under concurrent load
- Create reproducible load test scenarios
- Trigger change events across all three database types for comparison

## Target Users

- **CDC Developers**: Generate continuous change stream for CDC testing
- **Performance Engineers**: Load test databases with realistic transaction patterns
- **QA Engineers**: Create repeatable test workloads
- **Database Architects**: Compare database performance under identical workloads

## Dependencies

**Previous Versions:**
- v0.1.0 (Docker Container Management)
- v0.2.0 (Configuration Management System)
- v0.3.0 (Resource Monitoring)
- v0.4.0 (Database Metrics + TUI)
- v0.5.0 (Data Seeding) - Provides data for workload to operate on

**System Requirements:**
- Docker Engine 20.10+
- Rust 1.92+
- 4GB RAM minimum

## Success Criteria

- [ ] User can start a read-heavy workload with single command
- [ ] Workload generates configurable TPS (10, 100, 1000 TPS)
- [ ] Concurrent connections work correctly (1-100 connections)
- [ ] Workload runs for specified duration without errors
- [ ] Generated queries are valid and execute successfully
- [ ] Workload respects foreign key constraints when generating data
- [ ] Custom SQL scripts execute in sequence
- [ ] Workload statistics reported (total transactions, success rate, errors)
- [ ] Works across PostgreSQL, MySQL, and SQL Server

## Next Steps

**v0.7.0 - CDC Configuration Support** will introduce:
- Database-specific CDC enabling (PostgreSQL logical replication, MySQL binlog, SQL Server CDC/CT)
- CDC configuration helpers and validation
- Replication slot management
- Binlog position tracking
- CDC-specific monitoring integration
