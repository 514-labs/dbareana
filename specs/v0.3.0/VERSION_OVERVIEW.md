# Version 0.3.0 - Resource Monitoring

## Release Summary

This release adds comprehensive resource monitoring capabilities, enabling users to track CPU, memory, disk I/O, and network usage for each database container. Real-time and historical resource metrics provide visibility into database performance characteristics and resource consumption patterns.

## Key Features

- **CPU Usage Tracking**: Per-container CPU utilization percentage and CPU time
- **Memory Monitoring**: RAM consumption, memory limits, and usage percentages
- **Disk I/O Metrics**: Read/write operations, throughput, and latency
- **Network Traffic Monitoring**: Inbound/outbound bytes and packet counts
- **Historical Data**: Resource usage trends over time
- **Threshold Alerts**: Configurable warnings for resource limits

## Value Proposition

This release provides essential visibility into resource consumption, enabling users to:
- Understand the resource requirements of different database configurations
- Identify performance bottlenecks before they cause issues
- Compare resource efficiency across database types and versions
- Make informed decisions about container resource limits
- Detect resource leaks or anomalies during load testing
- Profile database behavior under different workloads

## Target Users

- **Performance Engineers**: Teams optimizing database configurations for resource efficiency
- **CDC Developers**: Engineers understanding the resource impact of CDC features
- **DevOps Engineers**: Teams planning container resource allocations
- **Database Architects**: Teams comparing resource footprints across database types

## Dependencies

**Previous Versions:**
- v0.1.0 (Docker Container Management + Rust CLI Foundation)
- v0.2.0 (Configuration Management System)

**System Requirements:**
- Docker Engine 20.10+ with stats API enabled
- Rust 1.92+
- 4GB RAM minimum

## Success Criteria

- [ ] CPU usage metrics updated in real-time (<1 second latency)
- [ ] Memory usage accurately reflects container consumption
- [ ] Disk I/O metrics captured for all database operations
- [ ] Network metrics track container traffic correctly
- [ ] Historical data retained for at least 1 hour (configurable)
- [ ] CLI can display current resource usage for all containers
- [ ] Resource metrics exportable in JSON format
- [ ] Minimal overhead: monitoring consumes <20MB RAM, <5% CPU

## Next Steps

**v0.4.0 - Database Metrics Collection + Real-Time TUI** will introduce:
- Database-specific performance metrics (query stats, connection pools, transaction rates)
- Real-time TUI dashboard for live metrics visualization
- Multi-pane interface with container status overview
- Interactive navigation between database instances
