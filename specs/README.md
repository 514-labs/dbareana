# dbarena Specifications

## Overview

This directory contains the versioned specifications for **dbarena**, a Docker-centric database simulation environment.

## Directory Structure

```
specs/
├── OVERVIEW.md                    # Master roadmap and project vision
├── IMPLEMENTATION_TRUTH.md        # Code-derived truth reference
├── DOCKER_OPTIMIZATION.md         # Performance and optimization strategies
├── README.md                      # This file
│
├── v0.1.0/                        # Foundation Release
│   ├── VERSION_OVERVIEW.md
│   ├── docker-container-management.md
│   └── rust-cli-foundation.md
│
├── v0.2.0/                        # Config + Init Scripts
│   ├── VERSION_OVERVIEW.md
│   └── configuration-management-system.md
│
├── v0.3.0/                        # Resource Monitoring
│   ├── VERSION_OVERVIEW.md
│   └── resource-monitoring.md
│
├── v0.4.0/                        # Database Metrics + TUI
│   ├── VERSION_OVERVIEW.md
│   ├── database-metrics-collection.md
│   └── real-time-tui.md
│
├── v0.5.0/                        # Data Seeding + Workload Generation
│   ├── VERSION_OVERVIEW.md
│   ├── data-seeding.md
│   └── workload-generation.md
│
├── v0.6.0/                        # Utilities & State Management
│   ├── VERSION_OVERVIEW.md
│   └── utilities-and-state.md
│
├── v0.7.0/                        # Database Docs + Search
│   ├── VERSION_OVERVIEW.md
│   └── database-documentation-search.md
│
├── v0.8.0/                        # Change Event Monitoring
│   ├── VERSION_OVERVIEW.md
│   └── change-event-monitoring.md
│
├── v1.0.0/                        # First Stable Release
│   ├── VERSION_OVERVIEW.md
│   └── release-notes.md
│
├── v1.1.0/                        # Benchmarking Suite (P1)
│   ├── VERSION_OVERVIEW.md
│   └── benchmarking-suite.md
│
├── v1.2.0/                        # Snapshot & Restore (P1)
│   ├── VERSION_OVERVIEW.md
│   └── snapshot-and-restore.md
│
├── v1.3.0/                        # Multi-Database Scenarios (P1)
│   ├── VERSION_OVERVIEW.md
│   └── multi-database-scenarios.md
│
├── v2.0.0/                        # OLAP Database Support (P2)
│   ├── VERSION_OVERVIEW.md
│   └── olap-database-support.md
│
├── v2.1.0/                        # Analytics Workloads (P2)
│   ├── VERSION_OVERVIEW.md
│   └── analytics-workloads.md
│
├── v2.2.0/                        # Export & Reporting (P2)
│   ├── VERSION_OVERVIEW.md
│   └── export-and-reporting.md
│
└── v2.3.0/                        # Configuration Profiles (P2)
    ├── VERSION_OVERVIEW.md
    └── configuration-profiles.md
```

## Release Priorities

### P0 - Core DB Simulation Capability (v0.1.0 - v1.0.0)
Complete end-to-end workflow from container setup through seeding, workloads, monitoring, and documentation.

**Key Milestone: v1.0.0** provides:
1. Spin up databases (PostgreSQL, MySQL, SQL Server)
2. Apply configuration + init scripts
3. Seed test data
4. Generate transactional workloads
5. Monitor performance in real-time
6. Access versioned database documentation

### P1 - Enhanced Testing (v1.1.0 - v1.3.0)
- Performance benchmarking
- State management (snapshots)
- Multi-database scenarios

### P2 - OLAP & Advanced Features (v2.0.0 - v2.3.0)
- Analytical database support
- Analytics workloads
- Export & reporting
- Team collaboration features

## Quick Start Guide

### Typical Workflow

```bash
# 1. Create database
dbarena create postgres --version 16

# 2. (Optional) Run init scripts
dbarena create postgres --init-script ./schema.sql

# 3. Seed data
dbarena seed --config seed-ecommerce.toml --container dbarena-postgres-16-xxx --size medium

# 4. Run workload
dbarena workload --container dbarena-postgres-16-xxx --pattern ecommerce --tps 100 --duration 300

# 5. Monitor in TUI
dbarena stats --multipane
```

## Performance Targets

- **Warm start**: <5 seconds to ready database
- **Cold start**: <30 seconds (includes image pull)
- **Memory**: 256MB per container (default)
- **Disk**: 250MB per database type (Alpine images)

See [DOCKER_OPTIMIZATION.md](./DOCKER_OPTIMIZATION.md) for complete performance strategies.

### Validation & Benchmarks

All performance targets are validated with comprehensive benchmarks:
- **29 automated benchmarks** covering all operations
- **CI/CD integration** for regression detection
- **Historical tracking** for trend analysis

See [BENCHMARKS.md](./BENCHMARKS.md) for complete suite or [BENCHMARK_QUICK_REFERENCE.md](./BENCHMARK_QUICK_REFERENCE.md) for quick start.

Run benchmarks:
```bash
./scripts/run_benchmarks.sh
```

## Technical Requirements

- **Docker Engine**: 20.10+
- **Rust**: 1.92+ (for building from source)
- **RAM**: 8GB recommended
- **Terminal**: 80x24 minimum (120x40 recommended for TUI)

## Supported Databases

### OLTP Databases (v0.1.0+)
- PostgreSQL: 13, 14, 15, 16
- MySQL: 5.7, 8.0, 8.4
- SQL Server: 2019, 2022

### OLAP Databases (v2.0.0+)
- ClickHouse
- Apache Druid
- DuckDB
- TimescaleDB

## Key Features

### Foundation (v0.1.0 - v0.2.0)
- Docker container management
- Multi-database support
- Interactive Rust CLI
- TOML/YAML configuration for env + init scripts

### Monitoring (v0.3.0 - v0.4.0)
- Resource monitoring (CPU, memory, disk, network)
- Database metrics (queries, connections, transactions)
- Real-time TUI dashboard
- Live log streaming

### Testing Capabilities (v0.5.0)
- Schema-aware data seeding
- Realistic data generation
- Workload patterns (read-heavy, write-heavy, balanced, CDC-focused)
- Concurrent connection simulation
- Custom SQL script execution
 
### Documentation & Search (v0.7.0 planned)
- Versioned official documentation packs per database
- Fast local search for LLMs and humans

## Spec Policy

- **Code is the source of truth.** When specs disagree with code, update specs or explicitly mark as Planned/Not Implemented.
- `specs/IMPLEMENTATION_TRUTH.md` reflects current CLI behavior and config schema.
- Each `VERSION_OVERVIEW.md` must declare status: Implemented / Partially Implemented / Planned.

## Documentation

Each version includes:
- **VERSION_OVERVIEW.md**: Release summary, status, success criteria
- **Feature specs**: Technical requirements and design details
- **CLI examples**: Must match current clap definitions

## Contributing

When adding new specifications:
1. Create version directory (e.g., `v2.4.0/`)
2. Add `VERSION_OVERVIEW.md` with standard sections
3. Add feature specification files
4. Update this README with new version
5. Reference related versions in dependencies section

## Implementation Status

- ✅ Specifications complete for v0.1.0 - v2.3.0
- 🔄 Implementation: TBD
- 📋 Testing: TBD

## License

[License information]
