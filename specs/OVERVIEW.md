# dbarena - Database Simulation Environment

## Project Overview

dbarena is a Docker-centric database simulation environment designed for rapid, lightweight testing and comparison of multiple databases and configurations. Built with Rust, it provides an interactive CLI and real-time TUI for spinning up database instances, running load tests, and profiling resource consumption.

## Vision & Goals

- **Rapid Iteration**: Spin up and tear down database instances in seconds
- **Multi-Database Testing**: Run multiple database configurations side-by-side
- **Real-Time Visibility**: Live monitoring of database performance and resource consumption
- **CDC Testing (Optional)**: Supported via generic tooling (no automated CDC configuration)
- **Load Testing**: Compare database performance under various workloads
- **Lightweight**: Minimal overhead, Docker-native, efficient resource usage

## Primary Use Cases

1. **CDC Solution Testing**: Test Change Data Capture implementations across different databases and versions
2. **Configuration Comparison**: Compare different configurations of the same database
3. **Database Migration**: Evaluate migration paths between database systems
4. **Performance Benchmarking**: Load test and compare OLTP/OLAP databases
5. **Version Compatibility**: Test application compatibility across database versions

## Release Roadmap

### Current Release Status

| Version | Status | Release Date | Description |
|---------|--------|--------------|-------------|
| v0.1.0 | ✅ Implemented | 2025-01 | Docker Container Management |
| v0.2.0 | ✅ Implemented | 2025-01 | Configuration + Init Scripts |
| v0.3.0 | ✅ Implemented | 2025-01 | Resource Monitoring |
| v0.4.0 | ✅ Implemented | 2026-01 | Database Metrics + Real-Time TUI |
| v0.5.0 | ✅ Implemented | 2026-02 | Data Seeding + Workload Generation |
| v0.6.0 | ✅ Implemented | 2026-02 | Utilities & State Management |
| v0.7.0 | 📋 Planned | 2026-Q2 | Database Docs + Search |
| v0.8.0 | 📋 Planned | 2026-Q3 | Change Event Monitoring |

### v0.1.0 - Docker Container Management ✅
**Released: January 2025**
- Docker container lifecycle (create, start, stop, restart, destroy)
- Multi-database support (PostgreSQL, MySQL, SQL Server)
- Version selection and custom configurations
- Health checking and readiness detection
- Volume management for persistence
- Network isolation and port management

### v0.2.0 - Configuration Management System ✅
**Released: January 2025**
- TOML-based configuration system
- Schema definitions with DDL generation
- Configuration validation
- Environment profiles and variables
- Initialization script management
- Template system for reusable configs

### v0.3.0 - Resource Monitoring ✅
**Released: January 2025**
- CPU usage tracking per container
- Memory consumption monitoring
- Disk I/O metrics
- Network traffic statistics
- Container resource limits enforcement

### v0.4.0 - Database Metrics + Real-Time TUI ✅
**Released: February 2025**
- Database-specific metrics collection (QPS, TPS, connections)
- Query execution statistics
- Real-time terminal user interface (TUI)
- Multi-pane dashboard with live updates
- Log streaming and visualization
- Interactive navigation between containers

### v0.5.0 - Data Seeding + Workload Generation ✅
**Status: Implementation Complete - February 2025**

**Data Seeding:**
- 11 built-in data generators (sequential, random, email, name, etc.)
- Foreign key resolution and relationship preservation
- Dependency-aware topological ordering
- Volume scaling (1K to 1M+ rows)
- Deterministic seeding for reproducibility
- Multi-database support (Postgres, MySQL, SQL Server)

**Workload Generation:**
- 10 built-in patterns (OLTP, E-commerce, OLAP, Time-Series, etc.)
- Concurrent connection simulation (1-100+ workers)
- Realistic CRUD operation generation
- Rate limiting with precise TPS control (10-1000+ TPS)
- Live progress monitoring with latency percentiles
- Custom workload configurations

**Commands:**
- `dbarena seed` - Populate databases with test data
- `dbarena workload` - Execute workload patterns

**Testing:**
- 130 tests passing (121 unit, 9 smoke tests)
- Performance validated (<60s for 100K rows, ±10% TPS accuracy)
- Manual E2E testing guide complete

### v0.6.0 - Utilities & State Management ✅
**Implemented: 2026-02**

- Exec/query utilities
- Snapshot management
- Volume management
- Network management
- Template import/export

### v0.7.0 - Database Docs + Search 📋
**Planned: 2026-Q2**

- Installable official documentation packs per DB + version
- Ultra-fast local search for LLMs and humans
- JSON output for programmatic consumption

### v0.8.0 - Change Event Monitoring 📋
**Planned: 2026-Q3**

- Real-time change event capture (requires external CDC setup)
- Event rate visualization, filtering, and export
- TUI integration for change events

---

## Feature Roadmap (Post v0.8.0)

### Future Versions (Priority 1)
**v1.1.0 - Benchmarking Suite**
- Standard benchmark execution (TPC-like)
- Custom benchmark definitions
- Result comparison across instances
- Historical performance tracking

**v1.2.0 - Snapshot & Restore**
- Save database states
- Quick restore to checkpoints
- Compare before/after scenarios
- A/B testing support

**v1.3.0 - Multi-Database Scenarios**
- Coordinated startup of database clusters
- Cross-database query support
- Data synchronization scenarios
- Failover simulation

### Future Versions (Priority 2)
**v2.0.0 - OLAP Database Support**
- ClickHouse, Apache Druid, DuckDB, TimescaleDB
- Columnar storage patterns
- Time-series workloads

**v2.1.0 - Analytics Workloads**
- Complex query benchmarks
- Data warehousing patterns
- Batch processing simulation
- Query optimization comparison

**v2.2.0 - Export & Reporting**
- Performance report generation
- Metrics export (Prometheus format)
- Comparison charts and graphs
- CI/CD integration support

**v2.3.0 - Configuration Profiles**
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
3. Database queries → Docker exec (psql/mysql/sqlcmd) → Metrics
4. Logs/metrics → Storage → Real-time display/export

### Technology Stack
- **Language**: Rust (CLI, TUI, orchestration)
- **Container Runtime**: Docker / Docker Compose
- **TUI Library**: ratatui or cursive
- **Database Access**: Docker exec via database CLI tools (psql, mysql, sqlcmd)
- **Metrics**: Custom collectors + optional Prometheus integration
- **Configuration**: TOML/YAML for declarative configs

## Database Support Strategy

### Initial Focus: OLTP Databases
Priority on transactional databases; CDC usage is optional and not auto-configured:
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
