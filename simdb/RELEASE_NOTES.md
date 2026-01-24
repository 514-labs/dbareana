# Release Notes

## v0.1.0 - Foundation Release (2026-01-22)

First release of dbarena with core container management functionality.

### Features

#### Container Lifecycle Management
- Create, start, stop, restart, and destroy database containers
- Support for multiple containers simultaneously
- Automatic container naming or custom names
- Container tracking with dbarena labels

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
- **Interactive Mode** - Visual menus with fuzzy search for all commands
- **Main Menu** - Run without any command for guided interface
- **Multi-Version Selection** - Create multiple versions of databases simultaneously
- **Batch Operations** - Select all containers and confirm-all-at-once for destroy
- Progress indicators for all operations
- Colored output for better UX
- Connection strings automatically generated
- Verbose logging modes (-v, -vv, -vvv)
- JSON output support (planned)

#### Commands
- `dbarena` - Interactive main menu (no command required)
- `dbarena create <databases> [-i]` - Create and start containers (use -i for interactive)
- `dbarena list [--all]` - List containers
- `dbarena start <container> [-i]` - Start stopped container (use -i to select from menu)
- `dbarena stop <container> [-i] [--timeout]` - Stop running container
- `dbarena restart <container> [-i]` - Restart container
- `dbarena destroy <container> [-i] [-y] [-v]` - Remove container (use -i for multi-select)
- `dbarena inspect <container> [-i]` - View details
- `dbarena logs <container> [-i] [-f] [--tail]` - View logs

All commands support `-i` flag for interactive mode with visual menus.

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
git clone https://github.com/yourusername/dbarena.git
cd dbarena
cargo build --release
./target/release/dbarena --version
```

#### Requirements
- Docker Engine running locally
- Rust 1.70+ (for building from source)

### Example Usage

```bash
# Interactive main menu (easiest way to start)
dbarena

# Interactive create - select databases and multiple versions
dbarena create -i

# Create a PostgreSQL database
dbarena create postgres

# Create multiple databases
dbarena create postgres mysql sqlserver

# Create with custom settings
dbarena create postgres --version 15 --name my-db --port 5433

# List all containers
dbarena list

# Stop a container interactively
dbarena stop -i

# Destroy multiple containers (select all option available)
dbarena destroy -i

# Destroy with confirmation skip
dbarena destroy my-db -y
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

Please report issues at: https://github.com/yourusername/dbarena/issues

---

Thank you for using dbarena!
