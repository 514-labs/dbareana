# Docker Optimization Strategy for simDB

## Overview

This document outlines strategies to ensure simDB's Docker-based database environments are lightweight and fast to set up, achieving the goal of database-ready state in <10 seconds.

## Performance Targets

- **Cold start** (no images cached): <30 seconds (includes image pull)
- **Warm start** (images cached): <5 seconds to ready state
- **Memory overhead**: <50MB per container baseline
- **Disk space**: <500MB per database type (compressed images)

## 1. Image Selection & Management

### Use Slim/Alpine Images Where Possible

**PostgreSQL:**
```dockerfile
# Standard: postgres:16 (~400MB)
# Optimized: postgres:16-alpine (~250MB)
postgres:16-alpine
```

**MySQL:**
```dockerfile
# Standard: mysql:8.0 (~600MB)
# Optimized: mysql:8.0-debian (~450MB) - Alpine not officially supported
mysql:8.0-debian
```

**SQL Server:**
```dockerfile
# Only standard available: mcr.microsoft.com/mssql/server:2022-latest (~1.5GB)
# No Alpine variant available
```

### Image Pre-Pulling Strategy

**Automatic Pre-Pull on First Run:**
```rust
async fn ensure_images_cached() -> Result<()> {
    let required_images = [
        "postgres:16-alpine",
        "postgres:15-alpine",
        "mysql:8.0-debian",
        "mysql:5.7-debian",
        "mcr.microsoft.com/mssql/server:2022-latest",
    ];

    // Pull in parallel
    let mut tasks = Vec::new();
    for image in required_images {
        if !is_image_cached(image).await? {
            println!("Pre-caching image: {}", image);
            tasks.push(pull_image_with_progress(image));
        }
    }

    futures::future::join_all(tasks).await;
    Ok(())
}
```

**CLI Command:**
```bash
# Pre-pull common images
simdb images pull --all
simdb images pull --database postgres

# List cached images
simdb images list

# Prune unused images
simdb images prune
```

### Custom Optimized Images (Optional)

For advanced users, provide pre-configured images:

```dockerfile
# simdb-postgres:16
FROM postgres:16-alpine

# Pre-configure for CDC
RUN echo "wal_level = logical" >> /usr/share/postgresql/postgresql.conf.sample && \
    echo "max_replication_slots = 10" >> /usr/share/postgresql/postgresql.conf.sample && \
    echo "max_wal_senders = 10" >> /usr/share/postgresql/postgresql.conf.sample

# Pre-install common extensions
RUN apk add --no-cache postgresql-contrib

# Reduce image size
RUN rm -rf /var/cache/apk/*
```

## 2. Container Startup Optimization

### Parallel Container Creation

```rust
pub async fn create_multiple_containers(configs: Vec<ContainerConfig>) -> Result<Vec<Container>> {
    // Create all containers in parallel
    let creation_tasks: Vec<_> = configs
        .into_iter()
        .map(|config| tokio::spawn(create_container(config)))
        .collect();

    // Wait for all to complete
    let results = futures::future::join_all(creation_tasks).await;

    // Start all containers in parallel
    let start_tasks: Vec<_> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .map(|container| tokio::spawn(start_container(container)))
        .collect();

    futures::future::join_all(start_tasks).await
}
```

### Fast Health Checks

**Optimized Health Check Strategy:**
```rust
pub struct HealthCheckConfig {
    // Start checking immediately (no delay)
    start_period: Duration::from_secs(0),

    // Check frequently at first
    interval: Duration::from_millis(250),

    // Short timeout for fast failure detection
    timeout: Duration::from_secs(2),

    // Only need 1 success to be healthy
    retries: 1,
}

// Lightweight health check commands
impl DatabaseHealthCheck {
    fn postgres_check() -> Vec<&'static str> {
        // Fast: just check if accepting connections
        vec!["pg_isready", "-U", "postgres"]
    }

    fn mysql_check() -> Vec<&'static str> {
        // Fast: ping without full authentication
        vec!["mysqladmin", "ping", "-h", "localhost"]
    }

    fn sqlserver_check() -> Vec<&'static str> {
        // Fast: simple SELECT 1
        vec!["/opt/mssql-tools/bin/sqlcmd", "-S", "localhost",
             "-U", "SA", "-P", "$SA_PASSWORD", "-Q", "SELECT 1", "-b"]
    }
}
```

### Lazy Initialization

**Defer Non-Critical Setup:**
```rust
pub async fn create_container_fast(config: &ContainerConfig) -> Result<Container> {
    // 1. Create container (fast)
    let container = docker.create_container(config).await?;

    // 2. Start container (fast)
    docker.start_container(&container.id).await?;

    // 3. Wait for healthy (fast - optimized health checks)
    wait_for_healthy(&container, Duration::from_secs(5)).await?;

    // Container is ready!
    // Defer these until actually needed:
    // - Creating databases (only create when user runs 'config deploy')
    // - Creating users (only when needed)
    // - Installing extensions (only when needed)

    Ok(container)
}
```

## 3. Resource Allocation

### Smart Default Limits

**Minimal Resource Allocation:**
```rust
pub fn default_resource_limits(db_type: DatabaseType) -> ResourceLimits {
    match db_type {
        DatabaseType::Postgres => ResourceLimits {
            memory: 256_000_000,      // 256MB (enough for testing)
            memory_swap: 512_000_000, // 512MB swap
            cpu_shares: 512,           // 50% of one core
            cpu_quota: 50_000,         // 50% CPU limit
        },
        DatabaseType::MySQL => ResourceLimits {
            memory: 512_000_000,      // 512MB (MySQL more memory-hungry)
            memory_swap: 1_024_000_000,
            cpu_shares: 512,
            cpu_quota: 50_000,
        },
        DatabaseType::SQLServer => ResourceLimits {
            memory: 2_048_000_000,    // 2GB minimum for SQL Server
            memory_swap: 2_048_000_000,
            cpu_shares: 1024,
            cpu_quota: 100_000,
        },
    }
}
```

**User Override:**
```bash
# Override defaults for specific needs
simdb create postgres --memory 1024 --cpu 2.0
```

### tmpfs for Ephemeral Data

**Use RAM for Temporary Data:**
```rust
pub fn configure_tmpfs_mounts() -> Vec<Mount> {
    vec![
        // Use tmpfs for /tmp (faster, no disk I/O)
        Mount {
            target: "/tmp",
            source: None,
            typ: Some(MountType::Tmpfs),
            tmpfs_options: Some(TmpfsOptions {
                size_bytes: 100_000_000, // 100MB
                mode: 0o1777,
            }),
        },

        // PostgreSQL: Use tmpfs for unlogged tables
        Mount {
            target: "/var/lib/postgresql/data/pg_stat_tmp",
            typ: Some(MountType::Tmpfs),
            tmpfs_options: Some(TmpfsOptions {
                size_bytes: 50_000_000, // 50MB
                mode: 0o700,
            }),
        },
    ]
}
```

## 4. Network Optimization

### Efficient Network Mode

**Use Bridge Network (Default):**
```rust
pub fn network_config() -> NetworkConfig {
    NetworkConfig {
        // Bridge network is fastest for most cases
        // Host network would be faster but less isolated
        mode: NetworkMode::Bridge,

        // Only expose necessary ports
        port_bindings: PortBindings::minimal(),

        // Disable IPv6 if not needed (faster)
        enable_ipv6: false,
    }
}
```

**Custom Network for Multi-Container:**
```rust
// Create shared network once for related containers
pub async fn create_simdb_network() -> Result<Network> {
    docker.create_network(CreateNetworkOptions {
        name: "simdb",
        driver: "bridge",
        internal: false,
        enable_ipv6: false,

        // Use default subnet (faster than custom)
        ipam: None,
    }).await
}
```

## 5. Configuration Management

### Embed Common Configurations

**Use Environment Variables Instead of Config Files:**
```rust
pub fn postgres_env_vars(config: &PostgresConfig) -> HashMap<String, String> {
    let mut env = HashMap::new();

    // Database initialization via env vars (no pg_hba.conf editing needed)
    env.insert("POSTGRES_USER".into(), config.user.clone());
    env.insert("POSTGRES_PASSWORD".into(), config.password.clone());
    env.insert("POSTGRES_DB".into(), config.database.clone());

    // CDC configuration via command line (no postgresql.conf editing)
    // Append to docker command: -c wal_level=logical -c max_replication_slots=10

    env
}

pub fn postgres_command(cdc_enabled: bool) -> Vec<String> {
    let mut cmd = vec!["postgres".to_string()];

    if cdc_enabled {
        cmd.extend([
            "-c".into(), "wal_level=logical".into(),
            "-c".into(), "max_replication_slots=10".into(),
            "-c".into(), "max_wal_senders=10".into(),
        ]);
    }

    // Performance optimizations for testing
    cmd.extend([
        "-c".into(), "fsync=off".into(),           // Dangerous for production, fast for testing
        "-c".into(), "synchronous_commit=off".into(),
        "-c".into(), "full_page_writes=off".into(),
        "-c".into(), "shared_buffers=128MB".into(),
    ]);

    cmd
}
```

### Configuration Templates

**Pre-Built Profiles:**
```toml
# ~/.simdb/profiles/postgres-cdc-fast.toml
[database]
type = "postgres"
version = "16"

[performance]
# Trading durability for speed (testing only!)
fsync = false
synchronous_commit = false
full_page_writes = false
wal_writer_delay = "200ms"
checkpoint_timeout = "15min"

[cdc]
enabled = true
wal_level = "logical"
max_replication_slots = 10
```

```bash
# Use pre-built profile
simdb create postgres --profile cdc-fast
```

## 6. Data Management

### Ephemeral by Default

**No Persistent Volumes Unless Requested:**
```rust
pub fn default_volume_config() -> VolumeConfig {
    VolumeConfig {
        // Use tmpfs (RAM) for data directory (fastest, lost on container stop)
        data_volume: VolumeType::Tmpfs {
            size_mb: 512,
        },

        // User must explicitly request persistence
        persistent: false,
    }
}
```

**User Request Persistence:**
```bash
# Explicitly enable persistence
simdb create postgres --persistent

# Or use profiles
simdb create postgres --profile persistent
```

### Lazy Volume Creation

```rust
pub async fn create_container_with_lazy_volumes(config: &ContainerConfig) -> Result<Container> {
    if config.persistent {
        // Create volume only if persistent mode requested
        create_named_volume(&config.name).await?;
    }
    // Otherwise, use tmpfs (no volume creation needed)

    create_container(config).await
}
```

## 7. Startup Parallelization

### Complete Startup Flow (Optimized)

```rust
pub async fn fast_startup_workflow(config: DatabaseConfig) -> Result<Container> {
    let start = Instant::now();

    // Phase 1: Image check (parallel for multiple databases)
    let image_check = ensure_image_cached(&config.image);

    // Phase 2: Port allocation (fast, no I/O)
    let port_alloc = find_available_port(config.preferred_port);

    // Phase 3: Container creation (while image check completes)
    let (image_ready, port) = tokio::join!(image_check, port_alloc);
    image_ready?;

    let container = docker.create_container(CreateContainerOptions {
        image: config.image,
        env: config.env_vars,
        ports: vec![port],
        resources: minimal_resources(),
        tmpfs: vec!["/tmp"], // Fast temporary storage
        command: optimized_command(&config),
    }).await?;

    // Phase 4: Start and health check (parallel if multiple containers)
    docker.start_container(&container.id).await?;

    // Phase 5: Wait for healthy (fast health checks)
    wait_for_healthy_fast(&container).await?;

    let elapsed = start.elapsed();
    println!("✓ Database ready in {:.2}s", elapsed.as_secs_f64());

    Ok(container)
}

async fn wait_for_healthy_fast(container: &Container) -> Result<()> {
    let mut attempts = 0;
    let max_attempts = 20; // 20 * 250ms = 5 seconds max

    loop {
        // Quick health check
        match execute_health_check(container).await {
            Ok(true) => return Ok(()),
            Ok(false) if attempts < max_attempts => {
                attempts += 1;
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            _ => return Err(anyhow!("Health check failed after 5 seconds")),
        }
    }
}
```

## 8. CLI Optimizations

### Smart Defaults

```bash
# Default: fast, ephemeral, minimal resources
simdb create postgres
# → postgres:16-alpine, 256MB RAM, tmpfs data, ready in ~3 seconds

# Explicit fast mode (same as default)
simdb create postgres --fast

# Persistent mode (slightly slower)
simdb create postgres --persistent
# → Adds volume creation: ~5 seconds

# Production-like mode (slower, durable)
simdb create postgres --durable
# → fsync=on, synchronous_commit=on, persistent: ~10 seconds
```

### Progress Indicators

```rust
pub async fn create_with_progress(config: &ContainerConfig) -> Result<Container> {
    let spinner = ProgressBar::new_spinner();

    spinner.set_message("Checking image cache...");
    ensure_image_cached(&config.image).await?;

    spinner.set_message("Creating container...");
    let container = create_container(config).await?;

    spinner.set_message("Starting database...");
    start_container(&container).await?;

    spinner.set_message("Waiting for healthy...");
    wait_for_healthy(&container).await?;

    spinner.finish_with_message("✓ Ready!");

    Ok(container)
}
```

## 9. Caching Strategies

### Connection Pool Reuse

```rust
pub struct ContainerManager {
    docker: Docker,
    // Keep Docker client connection alive
    connection_pool: ConnectionPool,
}

// Reuse Docker API connections
impl ContainerManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            docker: Docker::connect_with_defaults()?,
            connection_pool: ConnectionPool::new(5), // Reuse 5 connections
        })
    }
}
```

### Metadata Caching

```rust
pub struct ContainerCache {
    // Cache container metadata to avoid repeated Docker API calls
    metadata: Arc<RwLock<HashMap<String, ContainerMetadata>>>,
    ttl: Duration,
}

impl ContainerCache {
    pub async fn get_container_info(&self, id: &str) -> Result<ContainerInfo> {
        // Check cache first
        if let Some(cached) = self.metadata.read().get(id) {
            if cached.age() < self.ttl {
                return Ok(cached.info.clone());
            }
        }

        // Fetch from Docker API
        let info = self.fetch_from_docker(id).await?;

        // Update cache
        self.metadata.write().insert(id.into(), CachedMetadata {
            info: info.clone(),
            cached_at: Instant::now(),
        });

        Ok(info)
    }
}
```

## 10. Benchmarks & Validation

### Performance Tests

```rust
#[tokio::test]
async fn test_cold_start_performance() {
    // Ensure no images cached
    prune_all_images().await;

    let start = Instant::now();
    let container = create_postgres_container().await.unwrap();
    let elapsed = start.elapsed();

    // Should be < 30 seconds even with image pull
    assert!(elapsed.as_secs() < 30);
}

#[tokio::test]
async fn test_warm_start_performance() {
    // Ensure image cached
    pull_postgres_image().await.unwrap();

    let start = Instant::now();
    let container = create_postgres_container().await.unwrap();
    let elapsed = start.elapsed();

    // Should be < 5 seconds with cached image
    assert!(elapsed.as_secs() < 5);
}

#[tokio::test]
async fn test_parallel_startup() {
    let start = Instant::now();

    // Create 3 databases in parallel
    let containers = create_multiple_containers(vec![
        postgres_config(),
        mysql_config(),
        sqlserver_config(),
    ]).await.unwrap();

    let elapsed = start.elapsed();

    assert_eq!(containers.len(), 3);
    // Parallel should be faster than sequential
    // Sequential would be ~15s, parallel should be ~8s
    assert!(elapsed.as_secs() < 10);
}
```

## Implementation Priorities

### v0.1.0 (Foundation)
- ✅ Fast health checks
- ✅ Minimal resource allocation
- ✅ Ephemeral by default
- ✅ Parallel container creation

### v0.2.0 (Configuration)
- ✅ Environment variable configuration
- ✅ Configuration templates/profiles
- ✅ Command-line config passing

### Post v1.0.0
- Image pre-pulling on first run
- Custom optimized images
- Advanced caching strategies
- Startup benchmarks in CI

## Configuration Examples

### Fast Testing Profile
```toml
# ~/.simdb/profiles/fast.toml
[performance_mode]
preset = "fast"  # Prioritizes speed over durability

[resources]
memory_mb = 256
cpu_shares = 512

[storage]
type = "tmpfs"  # RAM-based, lost on stop
size_mb = 512

[durability]
fsync = false
synchronous_commit = false
```

### Durable Testing Profile
```toml
# ~/.simdb/profiles/durable.toml
[performance_mode]
preset = "durable"  # Prioritizes data safety

[resources]
memory_mb = 1024
cpu_shares = 1024

[storage]
type = "volume"  # Persistent disk
persistent = true

[durability]
fsync = true
synchronous_commit = true
```

## Summary

By implementing these optimizations, simDB achieves:

- **⚡ 3-5 second startup** (warm start with cached images)
- **💾 Minimal disk usage** (250MB per database type, tmpfs for ephemeral data)
- **🧠 Low memory footprint** (256MB per container default)
- **🚀 Parallel operations** (create multiple databases simultaneously)
- **⚙️ Smart defaults** (fast by default, durable on request)

The key insight: **Optimize for the 90% use case (fast, ephemeral testing) while allowing users to opt into durability when needed.**
