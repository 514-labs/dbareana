# Docker Container Management

## Feature Overview

Provides programmatic Docker container management for database instances, enabling users to spin up, monitor, and tear down database containers through the simDB CLI. This feature abstracts Docker complexity and provides a database-focused interface for container lifecycle operations.

**Status:** Implemented. CLI command name is `dbarena` (not `simdb`). See `specs/IMPLEMENTATION_TRUTH.md` for exact behavior.

**Performance Focus:** Optimized for rapid iteration with <5 second warm startup times. See [DOCKER_OPTIMIZATION.md](../DOCKER_OPTIMIZATION.md) for complete optimization strategy.

## Problem Statement

Manually managing database containers requires:
- Remembering complex Docker commands with database-specific parameters
- Tracking which ports are allocated to which database instances
- Ensuring proper volume mounting for data persistence
- Waiting for databases to be ready (health checking)
- Managing cleanup of containers and volumes

This creates friction when rapidly iterating on database tests or configurations. Users need a simplified interface that handles these concerns automatically.

## User Stories

**As a CDC developer**, I want to:
- Spin up a PostgreSQL 16 instance with a single command so I can immediately start testing my CDC connector
- Tear down and recreate a database instance to reset to a clean state without manually tracking container IDs
- Know when my database is ready to accept connections without manual polling

**As a database engineer**, I want to:
- Run multiple database versions side-by-side (e.g., PostgreSQL 15 and 16) without port conflicts
- Persist database data across container restarts for long-running tests
- Quickly clean up all test containers when my testing session is complete

## Technical Requirements

### Functional Requirements

**FR-1: Container Lifecycle Management**
- Support `create`, `start`, `stop`, `restart`, and `destroy` operations
- Generate unique container names to prevent collisions (e.g., `simdb-postgres-16-abc123`)
- Automatically assign available ports when defaults are in use
- Tag containers with simDB-specific labels for identification

**FR-2: Health Checking**
- Implement database-specific health checks:
  - PostgreSQL: `pg_isready` command
  - MySQL: `mysqladmin ping` command
  - SQL Server: SQL query check via `sqlcmd`
- Poll health status every 500ms with configurable timeout (default: 60 seconds)
- Report clear status: `starting`, `healthy`, `unhealthy`, `stopped`

**FR-3: Volume Management**
- Support two volume modes:
  - **Ephemeral**: Data is lost when container is destroyed (default)
  - **Persistent**: Data survives container destruction (named Docker volumes)
- Provide volume cleanup command to remove persistent volumes
- Prevent accidental deletion of persistent volumes without explicit confirmation

**FR-4: Multi-Database Support**
- PostgreSQL: Official `postgres:*-alpine` images (versions 13, 14, 15, 16) - optimized for size
- MySQL: Official `mysql:*-debian` images (versions 5.7, 8.0, 8.4) - slim variant
- SQL Server: Official `mcr.microsoft.com/mssql/server` images (2019, 2022) - no slim variant available

**Image Selection Strategy:**
- Prefer Alpine variants for minimal size (~250MB vs ~400MB)
- Use Debian slim variants when Alpine unavailable
- Pre-pull common images on first run for faster subsequent startups

**FR-5: Container Inspection**
- List all simDB-managed containers with status
- Show container details: name, database type, version, status, ports, uptime
- Stream container logs to stdout

### Non-Functional Requirements

**NFR-1: Performance**
- **Cold start** (image pull required): <30 seconds to ready state
- **Warm start** (image cached): <5 seconds to ready state
- Health checks should detect readiness within 5 seconds of database being ready
- Container destruction should complete within 3 seconds
- Memory overhead: <256MB per container (default configuration)
- Support parallel creation of multiple containers

**NFR-2: Reliability**
- Handle Docker daemon disconnections gracefully
- Retry transient Docker API failures (up to 3 retries with exponential backoff)
- Validate Docker availability before attempting operations

**NFR-3: Usability**
- Provide clear, actionable error messages
- Show progress indicators for long-running operations (image pulls, health checks)
- Default to safe operations (e.g., require confirmation for destructive actions)

## Architecture & Design

### Components

```
CLI Command
    ↓
Container Manager (Rust)
    ↓
Docker API Client (bollard crate)
    ↓
Docker Daemon
    ↓
Database Container
```

### Key Modules

**`container_manager.rs`**
- `ContainerManager` struct: Core orchestration
- Methods: `create()`, `start()`, `stop()`, `restart()`, `destroy()`, `list()`
- Handles Docker API communication via `bollard` crate

**`health_checker.rs`**
- `HealthChecker` trait: Database-agnostic health checking interface
- Implementations: `PostgresHealthChecker`, `MySQLHealthChecker`, `SQLServerHealthChecker`
- Async polling with timeout support

**`database_config.rs`**
- `DatabaseConfig` struct: Container configuration (image, version, ports, environment variables)
- Presets for common configurations (e.g., `DatabaseConfig::postgres_default()`)

### Data Flow

1. **Container Creation:**
   ```
   User command → Parse database type/version → Load config preset →
   Pull image (if needed) → Create container → Start container →
   Begin health check → Report ready/timeout
   ```

2. **Health Check Loop:**
   ```
   Start polling → Execute health command in container →
   Check exit code → Success? → Report healthy
                  ↓ Failure? → Retry after 500ms → Timeout? → Report unhealthy
   ```

## CLI Interface Design

### Commands

```bash
# Create and start a new database instance
simdb create <database> [options]
  --version <ver>       Database version (default: latest supported)
  --name <name>         Custom container name (default: auto-generated)
  --port <port>         Host port mapping (default: auto-assigned)
  --persistent          Enable data persistence across container restarts
  --env <KEY=VALUE>     Set environment variable (can be repeated)

# Examples:
simdb create postgres --version 16
simdb create mysql --version 8.0 --name my-test-db --persistent
simdb create sqlserver --version 2022 --port 1433

# Start an existing container
simdb start <name>

# Stop a running container (graceful shutdown)
simdb stop <name> [--timeout <seconds>]

# Restart a container
simdb restart <name>

# Destroy a container (and optionally its volume)
simdb destroy <name> [--remove-volume]

# List all simDB-managed containers
simdb list [--all]  # Include stopped containers

# Show container details
simdb inspect <name>

# Stream container logs
simdb logs <name> [--follow] [--tail <lines>]
```

### Example Usage Session

```bash
# Create a PostgreSQL 16 instance (warm start)
$ simdb create postgres --version 16
✓ Image postgres:16-alpine cached
✓ Creating container simdb-postgres-16-a3f9
✓ Starting database
✓ Waiting for healthy... (1.2s)
PostgreSQL ready in 2.8s

Connection Info:
  Host: localhost:5432
  Username: postgres
  Password: simdb
  Database: testdb

# List running containers
$ simdb list
NAME                      TYPE        VERSION  STATUS   PORTS          UPTIME
simdb-postgres-16-a3f9    postgres    16       healthy  5432->5432     30s

# Destroy when done
$ simdb destroy simdb-postgres-16-a3f9
✓ Container simdb-postgres-16-a3f9 destroyed (0.4s)

# First time setup (cold start with image pull)
$ simdb create mysql --version 8.0
⚠ Image mysql:8.0-debian not cached, pulling... (15.2s)
✓ Creating container simdb-mysql-8-b7e2
✓ Starting database
✓ Waiting for healthy... (3.1s)
MySQL ready in 18.7s

# Parallel creation (much faster than sequential)
$ simdb create postgres mysql sqlserver
Creating 3 containers in parallel...
  ✓ simdb-postgres-16-a3f9 ready (2.9s)
  ✓ simdb-mysql-8-b7e2 ready (3.2s)
  ✓ simdb-sqlserver-22-c9d4 ready (4.1s)
All containers ready in 4.3s (vs 10.2s sequential)
```

## Implementation Details

### Dependencies (Rust Crates)

```toml
[dependencies]
bollard = "0.16"              # Docker API client
tokio = { version = "1.36", features = ["full"] }
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"                # Error handling
thiserror = "1.0"             # Custom error types
tracing = "0.1"               # Logging
tracing-subscriber = "0.3"
```

### Docker Image Configuration

**PostgreSQL:**
```rust
DatabaseConfig {
    image: "postgres",
    version: "16-alpine",  // Use Alpine for smaller image size
    ports: vec![("5432", "5432")],
    environment: vec![
        ("POSTGRES_PASSWORD", "simdb"),
        ("POSTGRES_USER", "postgres"),
        ("POSTGRES_DB", "testdb"),
    ],
    health_check_command: vec!["pg_isready", "-U", "postgres"],

    // Performance optimizations for testing (not production!)
    command: vec![
        "postgres",
        "-c", "fsync=off",                    // Faster, less durable
        "-c", "synchronous_commit=off",       // Async commits
        "-c", "full_page_writes=off",         // Reduce WAL size
        "-c", "shared_buffers=128MB",         // Reasonable for testing
    ],

    // Resource limits (minimal for testing)
    memory_limit: 256_000_000,  // 256MB
    cpu_shares: 512,             // 50% of one core

    // Use tmpfs for ephemeral testing (lost on stop)
    tmpfs: vec![
        ("/tmp", TmpfsOptions { size_mb: 100 }),
        ("/var/lib/postgresql/data/pg_stat_tmp", TmpfsOptions { size_mb: 50 }),
    ],
}
```

**MySQL:**
```rust
DatabaseConfig {
    image: "mysql",
    version: "8.0-debian",  // Use Debian slim variant
    ports: vec![("3306", "3306")],
    environment: vec![
        ("MYSQL_ROOT_PASSWORD", "simdb"),
        ("MYSQL_DATABASE", "testdb"),
    ],
    health_check_command: vec!["mysqladmin", "ping", "-h", "localhost"],

    // Performance optimizations for testing
    command: vec![
        "--skip-log-bin",                     // Disable binlog initially (enable in v0.7.0)
        "--innodb-flush-log-at-trx-commit=2", // Faster commits
        "--innodb-buffer-pool-size=256M",     // Reasonable for testing
    ],

    // Resource limits
    memory_limit: 512_000_000,  // 512MB (MySQL more memory-hungry)
    cpu_shares: 512,

    tmpfs: vec![
        ("/tmp", TmpfsOptions { size_mb: 100 }),
    ],
}
```

**SQL Server:**
```rust
DatabaseConfig {
    image: "mcr.microsoft.com/mssql/server",
    version: "2022-latest",
    ports: vec![("1433", "1433")],
    environment: vec![
        ("ACCEPT_EULA", "Y"),
        ("SA_PASSWORD", "SimDB_Pass123!"),
    ],
    health_check_command: vec![
        "/opt/mssql-tools/bin/sqlcmd",
        "-S", "localhost",
        "-U", "SA",
        "-P", "SimDB_Pass123!",
        "-Q", "SELECT 1",
    ],
}
```

### Port Auto-Assignment Algorithm

```rust
async fn find_available_port(preferred: u16) -> Result<u16> {
    let start_port = preferred;
    let end_port = preferred + 100;

    for port in start_port..end_port {
        if is_port_available(port).await? {
            return Ok(port);
        }
    }

    Err(anyhow!("No available ports in range {}-{}", start_port, end_port))
}
```

### Container Labeling Strategy

All simDB containers include these labels:
- `simdb.managed=true`: Identifies simDB-managed containers
- `simdb.database_type=<type>`: postgres, mysql, sqlserver
- `simdb.database_version=<version>`: e.g., "16", "8.0", "2022"
- `simdb.created_at=<timestamp>`: UTC timestamp of creation

## Testing Strategy

### Unit Tests

- `test_database_config_presets()`: Verify preset configurations are valid
- `test_port_auto_assignment()`: Test port collision resolution
- `test_health_check_timeout()`: Ensure timeouts work correctly
- `test_container_naming()`: Verify unique name generation

### Performance Benchmarks

All performance targets are validated with automated benchmarks. See [BENCHMARKS.md](../BENCHMARKS.md) for complete suite.

**Critical Benchmarks:**
- `bench_cold_start_postgres()`: Validate <30s cold start
- `bench_warm_start_postgres()`: Validate <5s warm start
- `bench_warm_start_mysql()`: Validate <5s warm start
- `bench_parallel_container_creation()`: Validate parallel speedup
- `bench_memory_footprint_postgres()`: Validate <256MB memory
- `bench_memory_footprint_mysql()`: Validate <512MB memory
- `bench_health_check_detection()`: Validate <5s detection
- `bench_container_destruction()`: Validate <3s destruction

Run benchmarks:
```bash
./scripts/run_benchmarks.sh
```

### Integration Tests

- `test_postgres_lifecycle()`: Create, start, health check, stop, destroy PostgreSQL container
- `test_mysql_lifecycle()`: Full lifecycle test for MySQL
- `test_sqlserver_lifecycle()`: Full lifecycle test for SQL Server
- `test_persistent_volume()`: Verify data persists across container restarts
- `test_multiple_instances()`: Run 3 PostgreSQL instances simultaneously

### Manual Testing Scenarios

1. **Fresh Installation**: Install Docker, run `simdb create postgres`, verify success
2. **Port Conflict**: Start PostgreSQL on 5432, run `simdb create postgres`, verify auto-assignment
3. **Docker Not Running**: Stop Docker daemon, run any command, verify error message
4. **Image Pull**: Delete postgres image locally, create container, verify pull works
5. **Container Name Collision**: Create container with custom name, attempt to reuse name, verify error

## Documentation Requirements

- **README.md**: Getting started guide with installation instructions
- **CLI Reference**: Complete command documentation with examples
- **Database Support Matrix**: Supported databases, versions, and default configurations
- **Troubleshooting Guide**: Common errors and solutions
  - "Docker daemon is not running"
  - "Port already in use"
  - "Container failed to start"
  - "Health check timeout"

## Migration/Upgrade Notes

Not applicable - this is the initial release.

## Performance Optimization Strategies

### Image Management
```bash
# Pre-pull common images for faster subsequent startups
simdb images pull --all
simdb images pull postgres mysql

# List cached images
simdb images list

# Prune unused images
simdb images prune
```

### Fast vs Durable Modes
```bash
# Fast mode (default): ephemeral, optimized for speed
simdb create postgres --fast
# → tmpfs storage, fsync=off, 256MB RAM, ~3s startup

# Durable mode: persistent, production-like
simdb create postgres --durable
# → persistent volume, fsync=on, 1GB RAM, ~8s startup

# Persistent mode: middle ground
simdb create postgres --persistent
# → persistent volume, fast settings, ~5s startup
```

### Parallel Operations
```rust
// Internal implementation: parallel container creation
pub async fn create_multiple_containers(
    configs: Vec<ContainerConfig>
) -> Result<Vec<Container>> {
    // Create all containers concurrently
    let tasks: Vec<_> = configs
        .into_iter()
        .map(|config| tokio::spawn(create_container(config)))
        .collect();

    // Wait for all to complete
    let containers = futures::future::try_join_all(tasks).await?;

    Ok(containers)
}
```

### Resource Optimization
- **Default memory limits**: 256MB (PostgreSQL), 512MB (MySQL), 2GB (SQL Server)
- **tmpfs mounts**: Use RAM for /tmp and temporary database files
- **Fast health checks**: 250ms intervals, 2s timeout
- **Optimized commands**: Disable fsync, async commits for testing workloads

For complete optimization details, see [DOCKER_OPTIMIZATION.md](../DOCKER_OPTIMIZATION.md).

## Future Enhancements

- Support for additional databases (MariaDB, Oracle, CockroachDB)
- Custom simDB-optimized images with pre-configured CDC settings
- Custom health check commands via configuration
- Container auto-restart policies
- Multi-container network creation for database clusters
- Docker Compose integration for complex scenarios
- Automatic resource scaling based on workload
