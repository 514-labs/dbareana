# Version 1.3.0 - Multi-Database Scenarios

## Release Summary

Introduces coordinated multi-database testing capabilities, enabling complex scenarios involving database clusters, replication topologies, and cross-database interactions. Supports coordinated startup, data synchronization scenarios, and failover simulation.

## Key Features

- **Coordinated Startup**: Launch multiple databases as a group
- **Replication Topologies**: Primary-replica setups with automatic configuration
- **Cross-Database Queries**: Query across multiple database instances
- **Data Synchronization**: Test data flow between databases
- **Failover Simulation**: Trigger failovers and test recovery
- **Cluster Management**: Manage database groups as single units

## Value Proposition

Enables complex testing scenarios:
- Test primary-replica replication
- Validate CDC across database migrations (Postgres → MySQL)
- Simulate failover scenarios
- Test distributed applications
- Validate cross-database consistency

## Target Users

- **Platform Engineers**: Testing distributed database systems
- **SRE Teams**: Failover and disaster recovery testing
- **CDC Developers**: Cross-database change capture scenarios
- **Database Architects**: Replication topology validation

## Dependencies

- v1.0.0 (Complete CDC testing platform)
- v1.1.0 (Benchmarking suite)
- v1.2.0 (Snapshot & restore)

## Success Criteria

- [ ] User can create primary-replica cluster with one command
- [ ] Replication between instances configured automatically
- [ ] Failover simulation promotes replica to primary
- [ ] Cross-database queries work correctly
- [ ] Cluster operations (start, stop, restart) affect all members
- [ ] Replication lag visible in TUI

## Next Steps

**v2.0.0 - OLAP Database Support** will introduce support for analytical databases (ClickHouse, Druid, DuckDB, TimescaleDB).
