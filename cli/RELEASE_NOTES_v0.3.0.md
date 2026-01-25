# dbarena v0.3.0 Release Notes

**Release Date:** January 25, 2026
**Codename:** Performance, Snapshots, and Volumes

## 🎉 Overview

dbarena v0.3.0 introduces three major feature sets that significantly enhance database container management: real-time performance monitoring with an interactive TUI, container snapshots for state preservation, and comprehensive volume management for data persistence.

This release adds 3,444 lines of new code across 30 files while maintaining 100% backwards compatibility with v0.2.1.

## ✨ New Features

### 1. Performance Monitoring 📊

Real-time container metrics collection and visualization powered by Docker stats API.

**Features:**
- **Real-time metrics collection**: CPU, memory, network I/O, and block I/O
- **Interactive TUI**: Beautiful terminal UI with charts, gauges, and live updates (powered by Ratatui)
- **Multiple output modes**:
  - Simple text output for quick checks
  - Live updates with `--follow` (refresh every 2s)
  - Interactive TUI with `--tui` (charts, gauges, keyboard controls)
  - JSON output with `--json` for scripting
- **Multi-container monitoring**: Monitor all containers with `--all` flag
- **Rate calculations**: Automatic calculation of bytes/sec for network and disk I/O

**Commands:**
```bash
# One-time stats
dbarena stats <container>

# Live text updates
dbarena stats <container> --follow

# Interactive TUI dashboard
dbarena stats <container> --tui

# All containers in JSON
dbarena stats --all --json
```

**TUI Features:**
- Real-time CPU and memory gauges with color-coded thresholds
- 60-second history charts (sparklines) for CPU and memory
- Network I/O statistics with rates
- Block I/O statistics with rates
- Keyboard controls: q (quit), f (freeze), r (reset), h (help)

### 2. Container Snapshots 📸

Save and restore container state as Docker images with metadata.

**Features:**
- **Create snapshots** from running containers (auto-pause supported)
- **Restore containers** from snapshots with custom names and ports
- **Snapshot metadata**: Labels, timestamps, messages, database type
- **Full lifecycle management**: create, list, restore, delete, inspect
- **Docker integration**: Uses Docker commit API with proper labeling

**Commands:**
```bash
# Create snapshot
dbarena snapshot create <container> --name <snapshot-name> \
  --message "Description"

# List all snapshots
dbarena snapshot list [--json]

# Restore to new container
dbarena snapshot restore <snapshot> --name <new-name> --port <port>

# Delete snapshot
dbarena snapshot delete <snapshot> [--yes]

# Inspect details
dbarena snapshot inspect <snapshot> [--json]
```

**Use Cases:**
- Save database state before migrations
- Create testing environments from production snapshots
- Backup container state at specific points in time
- Quick rollback to known-good states

### 3. Volume Management 💾

Comprehensive volume lifecycle management for data persistence.

**Features:**
- **Named volumes**: Docker-managed volumes with automatic cleanup
- **Bind mounts**: Host directory mounts for local development
- **Volume CRUD**: Complete create, read, update, delete operations
- **Label filtering**: Only show dbarena-managed volumes by default
- **Integration**: Volume specifications in container config

**Commands:**
```bash
# Create volume
dbarena volume create <name> [--mount-path <path>]

# List volumes (dbarena-managed only by default)
dbarena volume list [--all] [--json]

# Delete volume
dbarena volume delete <name> [--force] [--yes]

# Inspect details
dbarena volume inspect <name> [--json]
```

**Configuration Support:**
```toml
[databases.postgres]
auto_volume = true
volume_path = "/var/lib/postgresql/data"

[[databases.postgres.volumes]]
name = "postgres-data"
path = "/var/lib/postgresql/data"
read_only = false

[[databases.postgres.bind_mounts]]
host = "./backups"
container = "/backups"
```

## 🔧 Configuration Enhancements

### New Configuration Sections

**Monitoring Configuration:**
```toml
[monitoring]
enabled = true
interval_seconds = 2
cpu_warning_threshold = 75.0
memory_warning_threshold = 80.0
```

**Snapshots Configuration:**
```toml
[snapshots]
auto_pause = true
storage_path = "~/.local/share/dbarena/snapshots"
max_snapshots_per_container = 10
```

**Database Volume Configuration:**
```toml
[databases.postgres]
auto_volume = true
volume_path = "/var/lib/postgresql/data"

[[databases.postgres.volumes]]
name = "pg-data"
path = "/var/lib/postgresql/data"
read_only = false
```

### Version Tracking
- Added `version` field to config schema for future migrations
- Current version: "0.3.0"

## 🧪 Testing

### Comprehensive Test Suite
- **80 unit tests** (added 17 new tests for v0.3.0)
- **Integration tests** for stats collection and container lifecycle
- **Smoke tests**: 16/16 passing (documented in SMOKE_TEST_RESULTS_v0.3.0.md)
- **Test coverage**:
  - Metrics calculation and rate computation
  - Byte/rate formatting functions
  - Snapshot metadata and lifecycle
  - Volume configuration and management
  - TUI helper functions
  - Config schema merging

### Test Results
```
Unit Tests:     80 passed, 0 failed
Smoke Tests:    16 passed, 0 failed
Coverage:       Comprehensive coverage of all new features
```

## 📦 Dependencies

### New Dependencies
- **ratatui 0.26**: Terminal UI framework for interactive dashboards
- **crossterm 0.27**: Cross-platform terminal manipulation

### Dependency Philosophy
- Minimal dependencies, maximum functionality
- Well-maintained, popular crates only
- Security-conscious dependency selection

## 🐛 Bug Fixes

### Snapshot Label Formatting
- **Issue**: Docker commit API rejected label syntax
- **Fix**: Changed from joined string to newline-separated LABEL instructions
- **Impact**: Snapshots now work correctly with Docker API
- **File**: `src/snapshot/storage.rs`

## 🔄 Backwards Compatibility

**100% Compatible** with dbarena v0.2.1:
- All existing commands work unchanged
- Config files are fully backwards compatible
- New config sections are optional
- Existing containers continue working
- No data migration required

## 📊 Statistics

### Code Changes
- **Files modified**: 12
- **Files added**: 18
- **Lines added**: 3,444
- **Lines removed**: 3
- **Modules created**: 3 (monitoring, snapshot, volume)

### Feature Breakdown
- **Performance Monitoring**: ~800 lines
- **Snapshots**: ~400 lines
- **Volumes**: ~300 lines
- **CLI Commands**: ~400 lines
- **Tests**: ~600 lines
- **Config & Integration**: ~300 lines

## 🚀 Migration Guide

### From v0.2.1 to v0.3.0

**No breaking changes!** Simply upgrade:

```bash
# Update binary
curl -sSL https://raw.githubusercontent.com/514-labs/dbareana/main/cli/install.sh | bash

# Verify version
dbarena --version
# Output: dbarena 0.3.0

# All existing functionality works as before
dbarena list
dbarena create postgres
# ... etc
```

**Optional: Enable new features in config:**

```toml
# Add to your config file (optional)
version = "0.3.0"

[monitoring]
enabled = true

[snapshots]
auto_pause = true
```

## 📚 Documentation

### New Documentation
- `SMOKE_TEST_RESULTS_v0.3.0.md`: Comprehensive test report
- Updated CLI help text for all new commands
- Extended config schema documentation

### Command Help
All new commands include detailed `--help` output:
```bash
dbarena stats --help
dbarena snapshot --help
dbarena volume --help
```

## 🎯 Use Cases

### Development Workflow
1. **Create development environment**: `dbarena create postgres --name dev-db`
2. **Monitor performance**: `dbarena stats dev-db --tui`
3. **Save state before changes**: `dbarena snapshot create dev-db --name before-migration`
4. **Make changes**: Run migrations, test features
5. **Rollback if needed**: `dbarena snapshot restore before-migration`

### Testing Workflow
1. **Create base container**: `dbarena create postgres --name test-base`
2. **Load test data**: Run seed scripts
3. **Create snapshot**: `dbarena snapshot create test-base --name seeded`
4. **Parallel testing**: Restore snapshot multiple times with different names
5. **Clean up**: `dbarena snapshot delete seeded --yes`

### Production Monitoring
1. **Monitor containers**: `dbarena stats --all --json > metrics.json`
2. **Process metrics**: Parse JSON for dashboards/alerts
3. **Track performance**: Compare metrics over time
4. **Identify issues**: High CPU/memory usage alerts

## 🏆 Highlights

### Interactive TUI
Professional terminal UI with:
- Real-time charts and gauges
- Color-coded thresholds (green/yellow/red)
- Keyboard navigation
- Pause/resume capability
- 60-second history visualization

### Snapshot System
Industrial-strength snapshots with:
- Metadata preservation
- Docker image integration
- Unique ID generation
- Timestamp tracking
- Message annotations

### Volume Management
Complete volume lifecycle with:
- Named volumes support
- Bind mounts support
- Read-only mounts
- Label-based filtering
- Config file integration

## 🔮 Future Roadmap

### Planned for v0.3.1 (Bulk Operations)
- Parallel container operations
- Multi-progress indicators
- Bulk start/stop/restart
- Performance: 3-5x faster than sequential

### Planned for v0.3.2 (Advanced Features)
- Custom network configuration
- Container connectivity
- Network isolation
- Container templates
- Template import/export

## 🙏 Acknowledgments

- **Ratatui team**: Excellent TUI framework
- **Bollard team**: Robust Docker API client
- **Community feedback**: Feature requests and bug reports

## 📝 Full Changelog

See commit history for detailed changes:
```bash
git log v0.2.1..v0.3.0
```

## 🔗 Links

- **Repository**: https://github.com/514-labs/dbareana
- **Issues**: https://github.com/514-labs/dbareana/issues
- **Documentation**: https://github.com/514-labs/dbareana/tree/main/docs

## 📄 License

MIT OR Apache-2.0

---

**Thank you for using dbarena!** 🎉

For questions, issues, or feature requests, please visit our GitHub repository.
