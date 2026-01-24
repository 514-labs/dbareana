# dbarena - Database Simulation Environment

A high-performance database simulation environment with Docker container management, designed for rapid testing and development.

## Status

✅ **v0.1.0 - Core Features Complete**

## Overview

dbarena provides instant database environments for testing, development, and experimentation. It manages Docker containers for popular databases with automatic health checking, resource management, and an intuitive CLI.

**Supported Databases:**
- PostgreSQL (default: v16)
- MySQL (default: v8.0)
- SQL Server (default: 2022-latest)

## Quick Start

```bash
# Build the project
cargo build --release

# Interactive mode - just run dbarena without any command!
./target/release/dbarena

# Or use specific commands:

# Create a PostgreSQL database (defaults to port 5432)
./target/release/dbarena create postgres

# Interactive create - select databases and multiple versions via menu
./target/release/dbarena create -i

# Create with custom settings
./target/release/dbarena create postgres --version 15 --name my-db --port 5433

# Create multiple databases at once
./target/release/dbarena create postgres mysql sqlserver

# List all containers
./target/release/dbarena list

# Stop a container
./target/release/dbarena stop my-db

# Start a stopped container
./target/release/dbarena start my-db

# Inspect container details
./target/release/dbarena inspect my-db

# View logs
./target/release/dbarena logs my-db

# Destroy a container
./target/release/dbarena destroy my-db

# Show help
./target/release/dbarena --help
```

## Features

### v0.1.0 (Current)

- ✅ **Container Lifecycle Management** - Create, start, stop, restart, destroy
- ✅ **Multi-Database Support** - PostgreSQL, MySQL, SQL Server
- ✅ **Interactive Mode** - Visual menus with multi-select for databases and versions
- ✅ **Multi-Version Support** - Create multiple versions of the same database simultaneously
- ✅ **Batch Operations** - Select all containers option and confirm-all-at-once for destroy
- ✅ **Health Checking** - Automatic readiness detection
- ✅ **Resource Management** - Memory limits, CPU shares, tmpfs mounts
- ✅ **CLI Interface** - Comprehensive command-line interface
- ✅ **Progress Indicators** - Visual feedback for all operations
- ✅ **Connection Strings** - Auto-generated connection info
- ✅ **Port Management** - Auto-assignment or custom ports
- ✅ **Container Tracking** - Labels for dbarena-managed containers

### Performance Targets

- Warm start (image cached): <5 seconds
- Cold start (image download): <30 seconds
- Health check detection: <5 seconds
- Container destruction: <3 seconds

## Installation

### From Source

```bash
git clone https://github.com/yourusername/dbarena.git
cd dbarena
cargo build --release
./target/release/dbarena --version
```

### Requirements

- Docker Engine running locally
- Rust 1.70+ (for building from source)

## Usage

### Main Interactive Menu

The easiest way to use dbarena is to run it without any command:

```bash
dbarena
```

This launches an interactive fuzzy-searchable menu where you can type to filter options or use arrow keys to navigate:

- 🚀 Create - Create and start new database container(s)
- 📋 List - List all containers
- ▶  Start - Start a stopped container
- ⏹  Stop - Stop a running container
- 🔄 Restart - Restart a container
- 🗑  Destroy - Remove container(s)
- 🔍 Inspect - View container details
- 📄 Logs - View container logs
- ❌ Exit - Quit dbarena

Simply select what you want to do, and dbarena will guide you through the rest!

### Creating Databases

#### Interactive Mode

The easiest way to create databases is with interactive mode:

```bash
# Launch interactive menu
dbarena create -i
```

This will guide you through:
1. **Multi-select database types** - Choose PostgreSQL, MySQL, and/or SQL Server
2. **Multi-select versions** - Pick multiple versions of each database to create in parallel
3. **Advanced options** (optional) - Configure memory limits, CPU shares, and persistence
4. **Confirmation** - Review your selections before proceeding

**Example**: Select PostgreSQL with versions 16, 15, and 14 to create three containers at once for version compatibility testing.

#### Command-Line Mode

```bash
# Create with defaults
dbarena create postgres

# Specify version
dbarena create postgres --version 15
dbarena create mysql --version 8.0
dbarena create sqlserver --version 2022-latest

# Custom name and port
dbarena create postgres --name my-test-db --port 5433

# With resource limits
dbarena create postgres --memory 512 --cpu-shares 512

# Persistent storage (survives container restarts)
dbarena create postgres --persistent

# Create multiple databases
dbarena create postgres mysql sqlserver
```

### Managing Containers

All management commands support interactive mode with `-i` flag:

```bash
# List all running containers
dbarena list

# List all containers (including stopped)
dbarena list --all

# Start a stopped container
dbarena start my-db
dbarena start -i              # Interactive: select from stopped containers

# Stop a running container
dbarena stop my-db
dbarena stop -i               # Interactive: select from running containers

# Stop with custom timeout (default: 10s)
dbarena stop my-db --timeout 30

# Restart a container
dbarena restart my-db
dbarena restart -i            # Interactive: select from running containers

# Inspect container details
dbarena inspect my-db
dbarena inspect -i            # Interactive: select any container

# View logs
dbarena logs my-db
dbarena logs -i               # Interactive: select any container

# Follow logs (like docker logs -f)
dbarena logs my-db --follow
dbarena logs -i --follow      # Interactive with follow mode

# Show last 50 lines
dbarena logs my-db --tail 50

# Destroy a container
dbarena destroy my-db
dbarena destroy -i            # Interactive: select all or multi-select containers

# Skip confirmation prompt
dbarena destroy my-db -y
dbarena destroy -i -y         # Interactive with auto-confirm all

# Also remove volumes
dbarena destroy my-db -v

# Interactive mode features:
# - "Select all containers" option for quick cleanup
# - "Confirm all deletions at once?" prompt for batch operations
# - Individual confirmations if needed
```

### Connection Examples

After creating a container, dbarena displays connection strings:

**PostgreSQL:**
```bash
psql -h localhost -p 54321 -U postgres -d testdb
# Connection string: postgres://postgres:postgres@localhost:54321/testdb
```

**MySQL:**
```bash
mysql -h localhost -P 54322 -u root -pmysql testdb
# Connection string: mysql://root:mysql@localhost:54322/testdb
```

**SQL Server:**
```bash
sqlcmd -S localhost,54323 -U sa -P 'YourStrong@Passw0rd'
```

### Logging

```bash
# Default (warnings and errors only, clean output)
dbarena create postgres

# Info level (shows operation details)
dbarena -v create postgres

# Debug level
dbarena -vv create postgres

# Trace level
dbarena -vvv create postgres

# Full debug (including Docker API)
dbarena -vvvv create postgres

# Quiet mode (errors only)
dbarena -q create postgres
```

## Development

### Running Tests

```bash
# Unit tests (no Docker required)
cargo test

# Integration tests (requires Docker)
cargo test --test integration -- --ignored

# Benchmarks (requires Docker)
cargo test --test benchmarks -- --ignored --nocapture

# All tests
cargo test --all
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy -- -D warnings

# Run all checks
cargo fmt --all -- --check && cargo clippy -- -D warnings && cargo test
```

### Project Structure

```
dbarena/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── error.rs             # Error types
│   ├── cli/
│   │   ├── mod.rs           # CLI structure
│   │   └── commands/        # Command implementations
│   ├── container/
│   │   ├── config.rs        # Container configuration
│   │   ├── docker_client.rs # Docker API client
│   │   ├── manager.rs       # Container lifecycle
│   │   └── models.rs        # Data models
│   └── health/
│       ├── checker.rs       # Health check trait
│       └── implementations.rs # DB-specific checkers
├── tests/
│   ├── integration/         # Integration tests
│   └── benchmarks/          # Performance benchmarks
└── .github/workflows/       # CI/CD
```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit a pull request

## Roadmap

### v0.2.0 - Configuration Management
- Configuration file support
- Environment variable profiles
- Custom database initialization scripts

### v0.3.0 - Resource Monitoring
- Real-time resource usage
- Container metrics
- Performance tracking

### v0.4.0 - Advanced Features
- Snapshot and restore
- Database seeding
- Workload generation

## Troubleshooting

### Docker Not Available

```
Error: Docker daemon not running or not accessible
```

**Solution:** Ensure Docker Desktop is running or Docker daemon is started.

### Port Already in Use

```
Error: Port 5432 is already in use
```

**Solution:** Specify a different port with `--port` or let dbarena auto-assign one.

### Container Won't Start

```bash
# Check Docker logs
dbarena logs <container-name>

# Inspect container details
dbarena inspect <container-name>

# Try verbose mode
dbarena -vvv start <container-name>
```

## License

MIT OR Apache-2.0

## Acknowledgments

Built with:
- [Bollard](https://github.com/fussybeaver/bollard) - Docker API client
- [Tokio](https://tokio.rs) - Async runtime
- [Clap](https://github.com/clap-rs/clap) - CLI framework
