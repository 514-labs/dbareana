# simDB - Database Simulation Environment

## Project Overview

simDB is a Docker-centric database simulation environment designed for rapid, lightweight testing and comparison of multiple databases and configurations. Built with Rust, it provides an interactive CLI and real-time TUI for spinning up database instances, running load tests, and profiling resource consumption.

## Vision & Goals

- **Rapid Iteration**: Spin up and tear down database instances in seconds
- **Multi-Database Testing**: Run multiple database configurations side-by-side
- **Real-Time Visibility**: Live monitoring of database performance and resource consumption
- **CDC Development**: Primary focus on Change Data Capture solution testing across databases and versions
- **Load Testing**: Compare database performance under various workloads
- **Lightweight**: Minimal overhead, Docker-native, efficient resource usage

## Primary Use Cases

1. **CDC Solution Testing**: Test Change Data Capture implementations across different databases and versions
2. **Configuration Comparison**: Compare different configurations of the same database
3. **Database Migration**: Evaluate migration paths between database systems
4. **Performance Benchmarking**: Load test and compare OLTP/OLAP databases
5. **Version Compatibility**: Test application compatibility across database versions

## Feature Roadmap (Prioritized)

### Phase 1: Core Infrastructure (P0)
1. **Docker Container Management**
   - Spin up/down database containers with preconfigured templates
   - Container lifecycle management (start, stop, restart, destroy)
   - Health checking and readiness detection
   - Volume management for data persistence/cleanup

2. **Rust CLI Foundation**
   - Interactive command-line interface
   - Database instance management commands
   - Configuration file support (YAML/TOML)
   - Template system for database configs

3. **Configuration Management System**
   - TOML-based database configuration definitions
   - Interactive CLI for configuration generation
   - DDL generation from configuration
   - Database-agnostic schema definitions
   - Version and variant management (e.g., Postgres 15 vs 16)
   - Configuration validation and templating

### Phase 2: Monitoring & Profiling (P0)
4. **Resource Monitoring**
   - CPU usage per container
   - Memory consumption tracking
   - Disk I/O metrics
   - Network traffic monitoring

5. **Database Metrics Collection**
   - Query execution statistics
   - Connection pool metrics
   - Transaction rates
   - Replication lag (where applicable)

6. **Real-Time TUI**
   - Multi-pane terminal interface
   - Live metrics dashboard
   - Container status overview
   - Log streaming per instance
   - Interactive navigation between instances

### Phase 3: CDC & Change Tracking (P0)
7. **CDC Configuration Support**
   - Enable binlog/WAL configurations
   - CDC-specific database settings
   - Replication setup helpers
   - Logical replication configuration

8. **Change Event Monitoring**
   - Track change events in real-time
   - Event rate visualization
   - Change stream inspection
   - Replication lag monitoring

### Phase 4: Load Testing & Simulation (P1)
9. **Workload Generation**
   - Built-in workload patterns (OLTP, read-heavy, write-heavy)
   - Custom SQL script execution
   - Concurrent connection simulation
   - Transaction replay capabilities

10. **Benchmarking Suite**
    - Standard benchmark execution (TPC-like)
    - Custom benchmark definitions
    - Result comparison across instances
    - Historical performance tracking

### Phase 5: Advanced Features (P1)
11. **Data Seeding**
    - Generate test data sets
    - Schema migration tools
    - Data volume scaling
    - Realistic data patterns

12. **Snapshot & Restore**
    - Save database states
    - Quick restore to checkpoints
    - Compare before/after scenarios
    - A/B testing support

13. **Multi-Database Scenarios**
    - Coordinated startup of database clusters
    - Cross-database query support
    - Data synchronization scenarios
    - Failover simulation

### Phase 6: OLAP & Analytics (P2)
14. **OLAP Database Support**
    - ClickHouse
    - Apache Druid
    - DuckDB
    - TimescaleDB

15. **Analytics Workloads**
    - Complex query benchmarks
    - Data warehousing patterns
    - Batch processing simulation
    - Query optimization comparison

### Phase 7: Enhanced Developer Experience (P2)
16. **Export & Reporting**
    - Performance report generation
    - Metrics export (Prometheus format)
    - Comparison charts and graphs
    - CI/CD integration support

17. **Configuration Profiles**
    - Saved environment configurations
    - Quick environment switching
    - Team-shared configurations
    - Environment versioning

## Technical Architecture (High-Level)

### Core Components
- **Rust CLI**: Command-line interface and orchestration engine
- **Docker Engine**: Container runtime and management
- **TUI Framework**: Terminal user interface (ratatui/crossterm)
- **Metrics Collector**: Resource and database metrics aggregation
- **Configuration Manager**: Template and profile management

### Data Flow
1. User commands → CLI parser → Docker API
2. Container events → Metrics collector → TUI display
3. Database queries → JDBC/native drivers → Metrics
4. Logs/metrics → Storage → Real-time display/export

### Technology Stack
- **Language**: Rust (CLI, TUI, orchestration)
- **Container Runtime**: Docker / Docker Compose
- **TUI Library**: ratatui or cursive
- **Database Drivers**: tokio-postgres, mysql_async, etc.
- **Metrics**: Custom collectors + optional Prometheus integration
- **Configuration**: TOML/YAML for declarative configs

## Database Support Strategy

### Initial Focus: OLTP Databases
Priority on transactional databases with CDC capabilities:
- **PostgreSQL** (13, 14, 15, 16): Logical replication, pg_replication_slot
- **MySQL** (5.7, 8.0, 8.4): Binlog-based CDC
- **SQL Server** (2019, 2022): Change Tracking and Change Data Capture (CDC)

### Version Strategy
- Support last 3-4 major versions per database
- Pin specific minor versions in templates
- Allow custom version specification
- Track version-specific features/limitations

### Future Expansion: OLAP
- Add analytical databases once OLTP foundation is solid
- Focus on columnar stores and time-series databases
- Enable hybrid OLTP/OLAP comparison scenarios

## Success Metrics

- **Startup Time**: Database instance ready in <5 seconds
- **Resource Overhead**: CLI/TUI uses <50MB RAM
- **Monitoring Latency**: Metrics updated <1 second delay
- **Developer Experience**: New environment setup in <2 minutes
- **Stability**: Containers run reliably for extended test periods

## Future Considerations

- Kubernetes deployment support
- Remote environment management
- Cloud database integration (RDS, Cloud SQL)
- Collaborative testing (shared environments)
- Plugin system for custom database support
- Integration with BI/visualization tools
- Cost estimation for cloud scenarios

## Non-Goals (Scope Boundaries)

- Not a production database management tool
- Not a replacement for database-native admin tools
- Not focused on database migration data transfer
- Not a general-purpose container orchestration platform
- Not designed for production monitoring/alerting
