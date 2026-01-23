# Release Notes

## v0.1.0 - Foundation Release (2026-01-22)

First release of simDB with core container management functionality.

### Features

#### Container Lifecycle Management
- Create, start, stop, restart, and destroy database containers
- Support for multiple containers simultaneously
- Automatic container naming or custom names
- Container tracking with simDB labels

#### Multi-Database Support
- PostgreSQL (default: v16)
- MySQL (default: v8.0)
- SQL Server (default: 2022-latest)
- Custom version specification

#### Health Checking
- Database-specific health checkers:
  - PostgreSQL: `pg_isready`
  - MySQL: `mysqladmin ping`
  - SQL Server: `sqlcmd SELECT 1`
- Automatic readiness detection with progress indicators
- Configurable timeouts (default: 60 seconds)
- 250ms check intervals for fast detection

#### Resource Management
- Memory limits
- CPU shares
- tmpfs mounts for improved performance
- Persistent volume support

#### CLI Interface
- Comprehensive command-line interface
- Progress indicators for all operations
- Colored output for better UX
- Connection strings automatically generated
- Verbose logging modes (-v, -vv, -vvv)
- JSON output support (planned)

#### Commands
- `simdb create <databases>` - Create and start containers
- `simdb list [--all]` - List containers
- `simdb start <container>` - Start stopped container
- `simdb stop <container> [--timeout]` - Stop running container
- `simdb restart <container>` - Restart container
- `simdb destroy <container> [-y] [-v]` - Remove container
- `simdb inspect <container>` - View details
- `simdb logs <container> [-f] [--tail]` - View logs

### Performance

Target performance (with cached images):
- Container creation: <5 seconds
- Health check detection: <5 seconds
- Container destruction: <3 seconds
- Cold start (image download): <30 seconds

### Testing

- Unit tests for core functionality
- Integration tests for container lifecycle
- Benchmarks for performance validation
- Test scripts for CI/CD

### Documentation

- Comprehensive README with examples
- Contributing guidelines
- Code documentation
- Troubleshooting guide
- Development setup instructions

### Known Limitations

- Port auto-assignment is random (doesn't check for actual availability)
- No configuration file support (coming in v0.2.0)
- No custom environment variables (coming in v0.2.0)
- No snapshot/restore functionality (planned)
- No resource monitoring (planned for v0.3.0)

### Breaking Changes

None (first release)

### Migration Guide

None (first release)

### Upgrading

First release - no upgrade needed.

### Installation

#### From Source
```bash
git clone https://github.com/yourusername/simdb.git
cd simdb
cargo build --release
./target/release/simdb --version
```

#### Requirements
- Docker Engine running locally
- Rust 1.70+ (for building from source)

### Example Usage

```bash
# Create a PostgreSQL database
simdb create postgres

# Create multiple databases
simdb create postgres mysql sqlserver

# Create with custom settings
simdb create postgres --version 15 --name my-db --port 5433

# List all containers
simdb list

# Stop a container
simdb stop my-db

# Destroy a container
simdb destroy my-db -y
```

### Contributors

- Claude Sonnet 4.5 <noreply@anthropic.com>

### What's Next

#### v0.2.0 - Configuration Management (Planned)
- Configuration file support (TOML/YAML)
- Environment variable profiles
- Custom database initialization scripts
- Environment variable injection

#### v0.3.0 - Resource Monitoring (Planned)
- Real-time resource usage
- Container metrics collection
- Performance tracking and history

See [ROADMAP.md](ROADMAP.md) for more details.

### Acknowledgments

Built with:
- [Bollard](https://github.com/fussybeaver/bollard) - Docker API client for Rust
- [Tokio](https://tokio.rs) - Asynchronous runtime
- [Clap](https://github.com/clap-rs/clap) - Command-line argument parser
- [Indicatif](https://github.com/console-rs/indicatif) - Progress indicators
- [Console](https://github.com/console-rs/console) - Styled terminal output

### Feedback

Please report issues at: https://github.com/yourusername/simdb/issues

---

Thank you for using simDB!
