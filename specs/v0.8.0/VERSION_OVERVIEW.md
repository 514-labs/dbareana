# Version 0.8.0 - Change Event Monitoring

## Release Summary

This release adds change event monitoring capabilities. Users can inspect change events in real-time, visualize event rates, filter change streams, and monitor replication lag. CDC configuration is assumed to be performed externally.

## Status

**Planned**

## Key Features

- **Real-Time Change Event Inspection**: View captured change events as they occur
- **Event Rate Visualization**: Track INSERT, UPDATE, DELETE rates per second
- **Change Stream Filtering**: Filter events by table, operation type, or time range
- **Replication Lag Monitoring**: Track CDC lag and identify bottlenecks
- **Event Format Display**: Show change events in readable format (JSON, table view)
- **TUI Integration**: Change event dashboard in the TUI alongside resource metrics

## Value Proposition

This release completes the dbarena CDC testing platform. Users can now:
- Verify CDC is capturing all change events correctly
- Measure CDC throughput and identify performance limits
- Debug CDC issues by inspecting individual change events
- Compare CDC performance across PostgreSQL, MySQL, and SQL Server
- Run complete end-to-end CDC tests: spin up → configure → seed → generate load → monitor changes
- Validate CDC connector behavior against database change streams

## Target Users

- **CDC Developers**: Validate CDC connectors capture all changes correctly
- **Platform Engineers**: Monitor CDC pipeline health and performance
- **QA Engineers**: Verify change capture behavior during integration tests
- **Database Architects**: Compare CDC performance across databases

## Dependencies

**Previous Versions:**
- v0.1.0 (Docker Container Management)
- v0.2.0 (Configuration + Init Scripts)
- v0.3.0 (Resource Monitoring)
- v0.4.0 (Database Metrics + TUI)
- v0.5.0 (Data Seeding + Workload Generation)
- v0.6.0 (Utilities & State Management)
- v0.7.0 (Database Docs + Search)

**Complete CDC Workflow Now Available (requires external CDC setup):**
1. Create database container (v0.1.0)
2. Apply configuration / init scripts (v0.2.0)
3. Configure CDC externally (DB-native or `dbarena exec/query`)
4. Seed data + generate workload (v0.5.0)
5. Monitor changes (v0.8.0)
6. View metrics in TUI (v0.4.0)

## Success Criteria

- [ ] User can view PostgreSQL logical replication changes in real-time
- [ ] User can view MySQL binlog events in real-time
- [ ] User can view SQL Server CDC changes in real-time
- [ ] Event rate metrics update every second
- [ ] Change events displayed in human-readable format
- [ ] Filtering works for table and operation type
- [ ] Replication lag displayed accurately
- [ ] TUI shows change event dashboard
- [ ] Zero change event loss during monitoring
- [ ] Works with all three database types

## Next Steps

**v1.0.0 - First Stable Release** will include:
- Complete documentation of end-to-end CDC testing workflow
- Performance benchmarks and tuning guide
- Production-ready stability and error handling
- Release notes consolidating all v0.x features
- Migration guide for upgrading from v0.x versions
