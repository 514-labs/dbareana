# Version 0.7.0 - CDC Configuration

## Release Summary

This release introduces Change Data Capture (CDC) configuration support for PostgreSQL, MySQL, and SQL Server. Users can now enable and configure CDC features through simDB, including PostgreSQL logical replication, MySQL binlog configuration, and SQL Server CDC/Change Tracking setup. The system provides guided configuration, validation, and monitoring integration.

## Key Features

- **PostgreSQL Logical Replication**: Create replication slots, configure WAL settings, manage logical replication
- **MySQL Binlog Configuration**: Enable binlog, set binlog format (ROW), configure retention
- **SQL Server CDC/CT**: Enable CDC or Change Tracking at database and table levels
- **Configuration Validation**: Verify CDC prerequisites and settings before enabling
- **Configuration Templates**: Pre-built CDC configurations for common scenarios
- **Monitoring Integration**: Track replication lag and CDC health in TUI (v0.4.0)

## Value Proposition

This release makes simDB a complete CDC testing environment. Users can now:
- Enable CDC on any supported database with a single command
- Test CDC connectors against correctly configured databases
- Avoid manual CDC configuration errors
- Switch between databases with CDC pre-configured
- Verify CDC configuration before starting tests
- Monitor CDC health alongside database metrics

## Target Users

- **CDC Developers**: Primary users testing CDC connectors and replication systems
- **Database Engineers**: Teams evaluating CDC solutions
- **QA Engineers**: Testing applications that rely on CDC
- **Platform Engineers**: Building data pipelines with CDC components

## Dependencies

**Previous Versions:**
- v0.1.0 (Docker Container Management)
- v0.2.0 (Configuration Management System)
- v0.3.0 (Resource Monitoring)
- v0.4.0 (Database Metrics + TUI)
- v0.5.0 (Data Seeding)
- v0.6.0 (Workload Generation)

**System Requirements:**
- Docker Engine 20.10+
- Rust 1.92+
- 4GB RAM minimum
- Database images with CDC support (PostgreSQL 10+, MySQL 5.7+, SQL Server 2017+)

## Success Criteria

- [ ] User can enable PostgreSQL logical replication with one command
- [ ] User can enable MySQL binlog with one command
- [ ] User can enable SQL Server CDC with one command
- [ ] Configuration validation prevents common mistakes
- [ ] PostgreSQL replication slots created successfully
- [ ] MySQL binlog format set to ROW correctly
- [ ] SQL Server CDC agents start automatically
- [ ] Configuration persists across container restarts
- [ ] CDC status visible in TUI dashboard
- [ ] Works for all supported database versions

## Next Steps

**v0.8.0 - Change Event Monitoring** will introduce:
- Real-time change event inspection
- Event rate visualization
- Change stream filtering and analysis
- Replication lag monitoring
- Complete end-to-end CDC testing workflow (config → seed → workload → monitor changes)
