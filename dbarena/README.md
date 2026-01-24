# dbarena - Database Simulation Environment

A high-performance database simulation environment with Docker container management, designed for rapid testing and development.

## Status

✅ **v0.2.0 - Configuration Management Complete**

## Overview

dbarena provides instant database environments for testing, development, and experimentation. It manages Docker containers for popular databases with automatic health checking, resource management, and an intuitive CLI.

**Supported Databases:**
- PostgreSQL (default: v16)
- MySQL (default: v8.0)
- SQL Server (default: 2022-latest)

## Installation

### Quick Install (macOS)

Install dbarena with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/514-labs/dbareana/main/dbarena/install.sh | bash
```

This will:
- Download the latest release binary
- Verify the checksum
- Install to `/usr/local/bin/dbarena`
- Make it available in your PATH

After installation, verify it works:
```bash
dbarena --version
```

### Manual Installation

Download the binary from [GitHub Releases](https://github.com/514-labs/dbareana/releases/latest):

```bash
# Download
curl -LO https://github.com/514-labs/dbareana/releases/download/v0.2.0/dbarena

# Make executable
chmod +x dbarena

# Move to PATH
sudo mv dbarena /usr/local/bin/

# Verify
dbarena --version
```

### Build from Source

```bash
git clone https://github.com/514-labs/dbareana.git
cd dbareana
cargo build --release
sudo cp target/release/dbarena /usr/local/bin/
```

### Uninstall

```bash
curl -sSL https://raw.githubusercontent.com/514-labs/dbareana/main/dbarena/uninstall.sh | bash
```

Or manually:
```bash
sudo rm /usr/local/bin/dbarena
```

## Quick Start

After [installation](#installation), you can start using dbarena:

```bash
# Interactive mode - just run dbarena without any command!
dbarena

# Or use specific commands:

# Create a PostgreSQL database (defaults to port 5432)
dbarena create postgres

# Interactive create - select databases and multiple versions via menu
dbarena create -i

# Create with custom settings
dbarena create postgres --version 15 --name my-db --port 5433

# Create multiple databases at once
dbarena create postgres mysql sqlserver

# List all containers
dbarena list

# Stop a container
dbarena stop my-db

# Start a stopped container
dbarena start my-db

# Inspect container details
dbarena inspect my-db

# View logs
dbarena logs my-db

# Destroy a container
dbarena destroy my-db

# Show help
dbarena --help
```

### v0.2.0 Features - Configuration & Init Scripts

```bash
# Generate example configuration
dbarena config init > dbarena.toml

# Use environment profiles
dbarena create postgres --profile dev

# Run initialization scripts
dbarena create postgres --init-script ./schema.sql

# Override environment variables
dbarena create postgres --env POSTGRES_DB=myapp

# Load from env file
dbarena create postgres --env-file .env.local

# Use configuration file
dbarena create postgres --config ./dbarena.toml

# Execute SQL on running container
dbarena exec my-postgres "SELECT * FROM users;"

# Execute SQL interactively
./target/release/dbarena exec -i --file ./query.sql
```

## Features

### v0.2.0 (Current)

**NEW - Configuration Management:**
- ✅ **Configuration Files** - TOML/YAML support for persistent settings
- ✅ **Environment Profiles** - Named profiles for dev/test/prod environments
- ✅ **Initialization Scripts** - Automatic SQL script execution on startup
- ✅ **Environment Variable Management** - Custom env vars with precedence control
- ✅ **Comprehensive Error Reporting** - Detailed script errors with line numbers and suggestions
- ✅ **Log Management** - Automatic logging of init script execution
- ✅ **SQL Execution** - Execute SQL scripts on running containers (inline or from file)
- ✅ **Config Utilities** - Validate, show, and initialize configuration files
- ✅ **Interactive Profile Selection** - Select profiles during interactive create

**Core Features (v0.1.0):**
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

### Configuration & Initialization (v0.2.0+)

#### Configuration Files

Create a `dbarena.toml` file in your project:

```toml
[databases.postgres.env]
POSTGRES_USER = "myapp"
POSTGRES_PASSWORD = "secret"
POSTGRES_DB = "myapp"

[databases.postgres]
init_scripts = ["./schema.sql", "./seed.sql"]
```

Then simply:

```bash
dbarena create postgres
# Uses config file automatically!
```

#### Environment Profiles

Define profiles for different environments:

```toml
[profiles.dev]
env = { LOG_LEVEL = "debug" }

[databases.postgres.profiles.dev]
env = { POSTGRES_DB = "myapp_dev" }

[databases.postgres.profiles.prod]
env = { POSTGRES_DB = "myapp_prod", POSTGRES_PASSWORD = "prod_secret" }
```

Use profiles:

```bash
dbarena create postgres --profile dev
dbarena create postgres --profile prod
```

#### Initialization Scripts

Automatically run SQL scripts on container creation:

```bash
# Single script
dbarena create postgres --init-script ./setup.sql

# Multiple scripts (run in order)
dbarena create postgres \
    --init-script ./schema.sql \
    --init-script ./seed.sql

# Or in config file
[databases.postgres]
init_scripts = ["./schema.sql", "./seed.sql"]
```

Scripts are executed after the database is healthy and output is logged to `~/.local/share/dbarena/logs/`.

#### Environment Variables

Override environment variables with precedence:

```bash
# CLI (highest priority)
dbarena create postgres --env POSTGRES_DB=custom_db

# From file
dbarena create postgres --env-file .env.local

# From config profile
dbarena create postgres --profile dev

# From config base
# [databases.postgres.env] in dbarena.toml
```

See [docs/CONFIGURATION.md](docs/CONFIGURATION.md) for complete reference.

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
│   │   ├── interactive.rs   # Interactive mode
│   │   └── commands/        # Command implementations
│   ├── config/              # NEW v0.2.0
│   │   ├── schema.rs        # Config data structures
│   │   ├── loader.rs        # File loading
│   │   ├── profile.rs       # Profile resolution
│   │   ├── merger.rs        # Config merging
│   │   └── validator.rs     # Validation
│   ├── container/
│   │   ├── config.rs        # Container configuration
│   │   ├── docker_client.rs # Docker API client
│   │   ├── manager.rs       # Container lifecycle
│   │   └── models.rs        # Data models
│   ├── health/
│   │   ├── checker.rs       # Health check trait
│   │   └── implementations.rs # DB-specific checkers
│   └── init/                # NEW v0.2.0
│       ├── copier.rs        # File copying to containers
│       ├── executor.rs      # Script execution & error parsing
│       └── logs.rs          # Log management
├── examples/                # NEW v0.2.0
│   ├── dbarena.toml         # Complete config example
│   ├── dbarena-minimal.toml # Minimal config
│   ├── profiles.toml        # Profile examples
│   └── scripts/             # Example SQL scripts
├── docs/                    # NEW v0.2.0
│   ├── CONFIGURATION.md     # Config reference
│   ├── INIT_SCRIPTS.md      # Init scripts guide
│   └── MIGRATION_V0.2.md    # Migration guide
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

### v0.2.0 - Configuration Management ✅ COMPLETE
- ✅ Configuration file support (TOML/YAML)
- ✅ Environment variable profiles
- ✅ Database initialization scripts with error reporting
- ✅ Comprehensive logging and debugging tools

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
