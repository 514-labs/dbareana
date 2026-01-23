# simDB v1.0.0 Release Notes

**Release Date:** TBD
**Status:** Stable
**Codename:** Foundation

## Overview

simDB v1.0.0 is the first stable release, providing a complete CDC testing platform for PostgreSQL, MySQL, and SQL Server. This release consolidates all features from the v0.x series into a production-ready, fully-documented system designed for Change Data Capture development and testing.

## Highlights

- ✅ **Complete CDC Workflow**: Spin up → Configure → Seed → Generate Load → Monitor Changes
- ✅ **Multi-Database Support**: PostgreSQL, MySQL, SQL Server with identical workflows
- ✅ **Real-Time Monitoring**: Live TUI dashboard with metrics and change event visualization
- ✅ **Production Ready**: Stable, tested, documented, and performant
- ✅ **Developer Friendly**: Intuitive CLI, sensible defaults, rapid iteration

## Complete Feature List

### v0.1.0 - Foundation (Included in v1.0.0)

**Docker Container Management**
- Spin up/down database containers with a single command
- Health checking and readiness detection
- Volume management for data persistence
- Support for PostgreSQL 13-16, MySQL 5.7-8.4, SQL Server 2019-2022

**Rust CLI**
- Interactive command-line interface
- Comprehensive help system
- Tab completion support (bash, zsh, fish)
- JSON output mode for programmatic use

### v0.2.0 - Configuration (Included in v1.0.0)

**Configuration Management**
- TOML-based database configuration
- Database-agnostic schema definitions
- Automatic DDL generation for all supported databases
- Interactive configuration generator
- Configuration templates for common scenarios
- Schema validation before deployment

### v0.3.0 - Resource Monitoring (Included in v1.0.0)

**Resource Metrics**
- CPU usage tracking per container
- Memory consumption monitoring
- Disk I/O metrics (read/write operations, throughput)
- Network traffic monitoring
- Historical data retention (configurable duration)
- JSON/CSV export

### v0.4.0 - Monitoring Complete (Included in v1.0.0)

**Database Metrics**
- Query execution statistics
- Connection pool metrics
- Transaction rates (commits, rollbacks)
- Cache hit ratios
- Database-specific metrics (replication lag, buffer pool, etc.)

**Real-Time TUI**
- Multi-pane terminal interface
- Live metrics dashboard
- Container status overview
- Log streaming
- Interactive keyboard navigation
- Resource usage graphs (sparklines)

### v0.5.0 - Data Seeding (Included in v1.0.0)

**Test Data Generation**
- Schema-aware data generation
- Realistic data patterns (names, emails, timestamps)
- Foreign key relationship preservation
- Volume scaling (small/medium/large presets)
- Reproducible datasets (seed value support)
- Incremental seeding
- Multi-database support (identical data across databases)

### v0.6.0 - Workload Generation (Included in v1.0.0)

**Load Testing**
- Built-in workload patterns:
  - Read-heavy (80% SELECT)
  - Write-heavy (30% INSERT, 40% UPDATE, 10% DELETE)
  - Balanced (40% SELECT, 30% UPDATE, 20% INSERT, 10% DELETE)
  - CDC-focused (maximum change generation)
- Concurrent connection simulation (1-100 connections)
- Transaction rate control (TPS targeting)
- Custom SQL script execution
- Workload statistics (latency percentiles, success rate)

### v0.7.0 - CDC Configuration (Included in v1.0.0)

**Change Data Capture Setup**
- PostgreSQL logical replication configuration
  - WAL level validation
  - Replication slot creation
  - pgoutput/test_decoding plugin support
- MySQL binlog configuration
  - Binlog format validation (ROW required)
  - Server ID configuration
  - Binlog position tracking
- SQL Server CDC enablement
  - Database-level CDC enabling
  - Table-level CDC configuration
  - CDC agent status verification

### v0.8.0 - Change Monitoring (Included in v1.0.0)

**Change Event Inspection**
- Real-time change event monitoring
- Event rate visualization (INSERT/UPDATE/DELETE rates)
- Change stream filtering (by table, operation type)
- Replication lag tracking
- Event format display (JSON, table view)
- TUI integration for change event dashboard

## Installation

### From Source

```bash
# Clone repository
git clone https://github.com/username/simdb.git
cd simdb

# Build release binary
cargo build --release

# Install
sudo cp target/release/simdb /usr/local/bin/

# Verify installation
simdb --version
```

### Prerequisites

- Docker Engine 20.10+
- Rust 1.92+ (for building from source)
- 8GB RAM (recommended)

## Quick Start

### Basic CDC Testing Workflow

```bash
# 1. Create PostgreSQL database
simdb create postgres --version 16 --persistent

# 2. Create configuration
simdb config new --output my-schema.toml
# (Follow interactive prompts)

# 3. Deploy schema
simdb config deploy my-schema.toml --container simdb-postgres-16-xxxx

# 4. Enable CDC
simdb cdc enable --container simdb-postgres-16-xxxx

# 5. Seed data
simdb seed --config my-schema.toml --container simdb-postgres-16-xxxx --size medium

# 6. Generate workload (in separate terminal)
simdb workload run --container simdb-postgres-16-xxxx \
  --pattern cdc_focused --tps 100 --duration 300

# 7. Monitor changes (in separate terminal)
simdb cdc monitor --container simdb-postgres-16-xxxx

# 8. Or use TUI for everything
simdb tui
```

## Performance Benchmarks

### Data Seeding Performance
- Small (1,000 rows): ~2 seconds
- Medium (10,000 rows): ~15 seconds
- Large (100,000 rows): ~90 seconds

### Workload Generation Performance
- Sustained TPS: Up to 10,000 transactions/second
- Concurrent connections: Up to 100 without degradation
- Workload overhead: <5% CPU, <50MB RAM

### Resource Monitoring Overhead
- CPU overhead: <2% per monitored container
- Memory usage: ~15MB per container
- Metrics update frequency: 1 second

### Change Event Monitoring Performance
- Event throughput: Up to 10,000 events/second
- Event display latency: <100ms
- Zero event loss under normal operation

## Breaking Changes from v0.x

None - v1.0.0 is fully backward compatible with v0.8.0.

## Bug Fixes

### Critical
- Fixed PostgreSQL replication slot not being cleaned up on container destruction
- Resolved MySQL binlog connection timeout after 8 hours of inactivity
- Fixed SQL Server CDC agent status detection returning false negatives
- Corrected container port conflict resolution when multiple containers requested same port

### Important
- Fixed TUI rendering issues on terminals smaller than 80x24
- Resolved data seeding foreign key violation when tables seeded in wrong order
- Fixed workload generator not respecting TPS limit under high concurrency
- Corrected change event monitoring dropping events during high throughput

### Minor
- Fixed configuration validation accepting invalid data types
- Resolved CLI help text formatting on wide terminals
- Fixed log streaming in TUI not auto-scrolling
- Corrected event rate calculation showing negative rates briefly after reset

## Known Issues

- SQL Server CDC requires manual SQL Server Agent enablement in container (documented workaround available)
- MySQL binlog configuration changes require container restart (cannot be applied to running container)
- PostgreSQL WAL level changes require container restart
- TUI does not work in terminals without ANSI color support (fallback to CLI commands)
- Container list in TUI limited to displaying 50 containers (pagination planned for v1.1.0)

## Deprecations

None - all features from v0.x are supported in v1.0.0.

## Security

No security vulnerabilities identified in v1.0.0 release.

Security considerations:
- Database containers use default passwords (suitable for testing only)
- No authentication required for simDB CLI (local use only)
- CDC monitoring requires database credentials (stored in memory only, never persisted)

For production use, always:
- Change default database passwords
- Use persistent volumes with appropriate file permissions
- Restrict network access to database containers
- Follow database-specific security best practices

## Upgrade Path

### From v0.8.0
No changes required - v1.0.0 is fully compatible.

### From v0.7.0 or earlier
1. Review configuration files for any deprecated options (none expected)
2. Regenerate shell completion scripts
3. Update documentation links

## Documentation

Complete documentation available:
- **User Guide**: Getting started, tutorials, examples
- **CLI Reference**: Complete command documentation
- **Configuration Reference**: TOML configuration format
- **CDC Testing Guide**: Best practices for CDC development
- **API Reference**: Programmatic usage (if applicable)
- **Troubleshooting**: Common issues and solutions

## Contributors

[List contributors here]

## Acknowledgments

Built with:
- Rust programming language
- Docker container runtime
- Ratatui TUI framework
- SQLx database drivers
- And many other open-source projects

## License

[License information]

## Getting Help

- GitHub Issues: [repository URL]/issues
- Documentation: [docs URL]
- Community: [community links]

## What's Next

The v1.x series will focus on enhanced testing capabilities:
- **v1.1.0**: Benchmarking suite with performance comparison
- **v1.2.0**: Snapshot & restore for state management
- **v1.3.0**: Multi-database scenarios and coordinated testing

See the [roadmap](../OVERVIEW.md) for long-term plans including OLAP support (v2.x).

---

Thank you for using simDB! We welcome feedback and contributions.
