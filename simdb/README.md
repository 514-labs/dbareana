# simDB - Database Simulation Environment

A high-performance database simulation environment with Docker container management, designed for rapid testing and development.

## Status

✅ **v0.1.0 - Core Features Complete**

## Overview

simDB provides instant database environments for testing, development, and experimentation. It manages Docker containers for popular databases with automatic health checking, resource management, and an intuitive CLI.

**Supported Databases:**
- PostgreSQL (default: v16)
- MySQL (default: v8.0)
- SQL Server (default: 2022-latest)

## Quick Start

```bash
# Build the project
cargo build --release

# Create a PostgreSQL database (defaults to port 5432)
./target/release/simdb create postgres

# Create with custom settings
./target/release/simdb create postgres --version 15 --name my-db --port 5433

# Create multiple databases at once
./target/release/simdb create postgres mysql sqlserver

# List all containers
./target/release/simdb list

# Stop a container
./target/release/simdb stop my-db

# Start a stopped container
./target/release/simdb start my-db

# Inspect container details
./target/release/simdb inspect my-db

# View logs
./target/release/simdb logs my-db

# Destroy a container
./target/release/simdb destroy my-db

# Show help
./target/release/simdb --help
```

## Features

### v0.1.0 (Current)

- ✅ **Container Lifecycle Management** - Create, start, stop, restart, destroy
- ✅ **Multi-Database Support** - PostgreSQL, MySQL, SQL Server
- ✅ **Health Checking** - Automatic readiness detection
- ✅ **Resource Management** - Memory limits, CPU shares, tmpfs mounts
- ✅ **CLI Interface** - Comprehensive command-line interface
- ✅ **Progress Indicators** - Visual feedback for all operations
- ✅ **Connection Strings** - Auto-generated connection info
- ✅ **Port Management** - Auto-assignment or custom ports
- ✅ **Container Tracking** - Labels for simDB-managed containers

### Performance Targets

- Warm start (image cached): <5 seconds
- Cold start (image download): <30 seconds
- Health check detection: <5 seconds
- Container destruction: <3 seconds

## Installation

### From Source

```bash
git clone https://github.com/yourusername/simdb.git
cd simdb
cargo build --release
./target/release/simdb --version
```

### Requirements

- Docker Engine running locally
- Rust 1.70+ (for building from source)

## Usage

### Creating Databases

```bash
# Create with defaults
simdb create postgres

# Specify version
simdb create postgres --version 15
simdb create mysql --version 8.0
simdb create sqlserver --version 2022-latest

# Custom name and port
simdb create postgres --name my-test-db --port 5433

# With resource limits
simdb create postgres --memory 512 --cpu-shares 512

# Persistent storage (survives container restarts)
simdb create postgres --persistent

# Create multiple databases
simdb create postgres mysql sqlserver
```

### Managing Containers

```bash
# List all running containers
simdb list

# List all containers (including stopped)
simdb list --all

# Start a stopped container
simdb start my-db

# Stop a running container
simdb stop my-db

# Stop with custom timeout (default: 10s)
simdb stop my-db --timeout 30

# Restart a container
simdb restart my-db

# Inspect container details
simdb inspect my-db

# View logs
simdb logs my-db

# Follow logs (like docker logs -f)
simdb logs my-db --follow

# Show last 50 lines
simdb logs my-db --tail 50

# Destroy a container
simdb destroy my-db

# Skip confirmation prompt
simdb destroy my-db -y

# Also remove volumes
simdb destroy my-db -v
```

### Connection Examples

After creating a container, simDB displays connection strings:

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
# Default (info level)
simdb create postgres

# Debug level
simdb -v create postgres

# Trace level
simdb -vv create postgres

# Full debug (including Docker API)
simdb -vvv create postgres

# Quiet mode (errors only)
simdb -q create postgres
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
simdb/
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

**Solution:** Specify a different port with `--port` or let simDB auto-assign one.

### Container Won't Start

```bash
# Check Docker logs
simdb logs <container-name>

# Inspect container details
simdb inspect <container-name>

# Try verbose mode
simdb -vvv start <container-name>
```

## License

MIT OR Apache-2.0

## Acknowledgments

Built with:
- [Bollard](https://github.com/fussybeaver/bollard) - Docker API client
- [Tokio](https://tokio.rs) - Async runtime
- [Clap](https://github.com/clap-rs/clap) - CLI framework
