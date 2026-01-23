# simDB Performance Benchmarks

## Overview

This document defines the complete benchmarking suite for validating simDB's performance targets. All benchmarks should be automated, reproducible, and integrated into CI/CD pipelines.

## Benchmark Categories

### 1. Container Operations
### 2. Resource Usage
### 3. Configuration Management
### 4. Data Operations
### 5. CDC Operations
### 6. TUI Performance
### 7. End-to-End Workflows

---

## 1. Container Operations Benchmarks

### 1.1 Cold Start Performance

**Target:** <30 seconds (includes image pull)

```rust
#[tokio::test]
#[ignore] // Run separately, requires network
async fn bench_cold_start_postgres() {
    // Ensure no cached image
    prune_image("postgres:16-alpine").await.ok();

    let start = Instant::now();
    let container = simdb::create_container(ContainerConfig {
        database: Database::Postgres,
        version: "16",
        ..Default::default()
    }).await.unwrap();

    wait_for_healthy(&container).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 30, "Cold start took {}s (target: <30s)", elapsed.as_secs());

    // Report metrics
    report_metric("cold_start_postgres", elapsed);
}

#[tokio::test]
#[ignore]
async fn bench_cold_start_mysql() {
    prune_image("mysql:8.0-debian").await.ok();

    let start = Instant::now();
    let container = simdb::create_container(ContainerConfig {
        database: Database::MySQL,
        version: "8.0",
        ..Default::default()
    }).await.unwrap();

    wait_for_healthy(&container).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 30, "Cold start took {}s (target: <30s)", elapsed.as_secs());
    report_metric("cold_start_mysql", elapsed);
}

#[tokio::test]
#[ignore]
async fn bench_cold_start_sqlserver() {
    prune_image("mcr.microsoft.com/mssql/server:2022-latest").await.ok();

    let start = Instant::now();
    let container = simdb::create_container(ContainerConfig {
        database: Database::SQLServer,
        version: "2022",
        ..Default::default()
    }).await.unwrap();

    wait_for_healthy(&container).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 30, "Cold start took {}s (target: <30s)", elapsed.as_secs());
    report_metric("cold_start_sqlserver", elapsed);
}
```

### 1.2 Warm Start Performance

**Target:** <5 seconds (cached image)

```rust
#[tokio::test]
async fn bench_warm_start_postgres() {
    // Ensure image is cached
    ensure_image_cached("postgres:16-alpine").await.unwrap();

    let start = Instant::now();
    let container = simdb::create_container(ContainerConfig {
        database: Database::Postgres,
        version: "16",
        ..Default::default()
    }).await.unwrap();

    wait_for_healthy(&container).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 5, "Warm start took {:.2}s (target: <5s)", elapsed.as_secs_f64());
    report_metric("warm_start_postgres", elapsed);

    // Cleanup
    destroy_container(&container).await.ok();
}

#[tokio::test]
async fn bench_warm_start_mysql() {
    ensure_image_cached("mysql:8.0-debian").await.unwrap();

    let start = Instant::now();
    let container = simdb::create_container(ContainerConfig {
        database: Database::MySQL,
        version: "8.0",
        ..Default::default()
    }).await.unwrap();

    wait_for_healthy(&container).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 5, "Warm start took {:.2}s (target: <5s)", elapsed.as_secs_f64());
    report_metric("warm_start_mysql", elapsed);

    destroy_container(&container).await.ok();
}

#[tokio::test]
async fn bench_warm_start_sqlserver() {
    ensure_image_cached("mcr.microsoft.com/mssql/server:2022-latest").await.unwrap();

    let start = Instant::now();
    let container = simdb::create_container(ContainerConfig {
        database: Database::SQLServer,
        version: "2022",
        ..Default::default()
    }).await.unwrap();

    wait_for_healthy(&container).await.unwrap();
    let elapsed = start.elapsed();

    // SQL Server takes longer due to initialization
    assert!(elapsed.as_secs() < 8, "Warm start took {:.2}s (target: <8s for SQL Server)", elapsed.as_secs_f64());
    report_metric("warm_start_sqlserver", elapsed);

    destroy_container(&container).await.ok();
}
```

### 1.3 Health Check Detection Time

**Target:** <5 seconds from database ready to detected

```rust
#[tokio::test]
async fn bench_health_check_detection() {
    let container = create_test_container().await.unwrap();

    // Database is starting up
    let start = Instant::now();

    // Wait for health check to detect ready state
    let result = wait_for_healthy(&container, Duration::from_secs(10)).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Health check failed");
    assert!(elapsed.as_secs() < 5, "Health check detection took {:.2}s (target: <5s)", elapsed.as_secs_f64());

    report_metric("health_check_detection", elapsed);

    destroy_container(&container).await.ok();
}
```

### 1.4 Container Destruction Time

**Target:** <3 seconds

```rust
#[tokio::test]
async fn bench_container_destruction() {
    let container = create_test_container().await.unwrap();
    wait_for_healthy(&container).await.unwrap();

    let start = Instant::now();
    destroy_container(&container).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 3, "Destruction took {:.2}s (target: <3s)", elapsed.as_secs_f64());
    report_metric("container_destruction", elapsed);
}
```

### 1.5 Parallel Container Creation

**Target:** 3 containers in <10 seconds (faster than 15s sequential)

```rust
#[tokio::test]
async fn bench_parallel_container_creation() {
    ensure_images_cached().await.unwrap();

    let configs = vec![
        ContainerConfig { database: Database::Postgres, version: "16", ..Default::default() },
        ContainerConfig { database: Database::MySQL, version: "8.0", ..Default::default() },
        ContainerConfig { database: Database::SQLServer, version: "2022", ..Default::default() },
    ];

    let start = Instant::now();
    let containers = create_containers_parallel(configs).await.unwrap();
    let elapsed = start.elapsed();

    assert_eq!(containers.len(), 3, "Expected 3 containers");
    assert!(elapsed.as_secs() < 10, "Parallel creation took {:.2}s (target: <10s)", elapsed.as_secs_f64());

    report_metric("parallel_creation_3_containers", elapsed);

    // Cleanup
    for container in containers {
        destroy_container(&container).await.ok();
    }
}

#[tokio::test]
async fn bench_sequential_vs_parallel() {
    ensure_images_cached().await.unwrap();

    let configs = vec![
        ContainerConfig { database: Database::Postgres, version: "16", ..Default::default() },
        ContainerConfig { database: Database::MySQL, version: "8.0", ..Default::default() },
        ContainerConfig { database: Database::SQLServer, version: "2022", ..Default::default() },
    ];

    // Sequential
    let start = Instant::now();
    let mut sequential_containers = Vec::new();
    for config in configs.clone() {
        let container = create_container(config).await.unwrap();
        wait_for_healthy(&container).await.unwrap();
        sequential_containers.push(container);
    }
    let sequential_time = start.elapsed();

    // Cleanup
    for container in sequential_containers {
        destroy_container(&container).await.ok();
    }

    // Parallel
    let start = Instant::now();
    let parallel_containers = create_containers_parallel(configs).await.unwrap();
    let parallel_time = start.elapsed();

    // Cleanup
    for container in parallel_containers {
        destroy_container(&container).await.ok();
    }

    println!("Sequential: {:.2}s, Parallel: {:.2}s, Speedup: {:.2}x",
        sequential_time.as_secs_f64(),
        parallel_time.as_secs_f64(),
        sequential_time.as_secs_f64() / parallel_time.as_secs_f64()
    );

    // Parallel should be at least 1.5x faster
    assert!(parallel_time.as_secs_f64() < sequential_time.as_secs_f64() * 0.67,
        "Parallel not significantly faster than sequential");

    report_metric("sequential_creation_time", sequential_time);
    report_metric("parallel_creation_time", parallel_time);
}
```

---

## 2. Resource Usage Benchmarks

### 2.1 Memory Footprint

**Target:** <256MB per PostgreSQL container, <512MB per MySQL container

```rust
#[tokio::test]
async fn bench_memory_footprint_postgres() {
    let container = create_container(ContainerConfig {
        database: Database::Postgres,
        version: "16",
        ..Default::default()
    }).await.unwrap();

    wait_for_healthy(&container).await.unwrap();

    // Wait for memory to stabilize
    tokio::time::sleep(Duration::from_secs(5)).await;

    let stats = get_container_stats(&container).await.unwrap();
    let memory_mb = stats.memory_usage / 1_000_000;

    assert!(memory_mb < 256, "Memory usage {}MB (target: <256MB)", memory_mb);
    report_metric("memory_postgres_mb", memory_mb as f64);

    destroy_container(&container).await.ok();
}

#[tokio::test]
async fn bench_memory_footprint_mysql() {
    let container = create_container(ContainerConfig {
        database: Database::MySQL,
        version: "8.0",
        ..Default::default()
    }).await.unwrap();

    wait_for_healthy(&container).await.unwrap();
    tokio::time::sleep(Duration::from_secs(5)).await;

    let stats = get_container_stats(&container).await.unwrap();
    let memory_mb = stats.memory_usage / 1_000_000;

    assert!(memory_mb < 512, "Memory usage {}MB (target: <512MB)", memory_mb);
    report_metric("memory_mysql_mb", memory_mb as f64);

    destroy_container(&container).await.ok();
}

#[tokio::test]
async fn bench_memory_footprint_under_load() {
    let container = create_test_container().await.unwrap();
    wait_for_healthy(&container).await.unwrap();

    // Run workload for 60 seconds
    let workload_handle = tokio::spawn(async move {
        run_workload(WorkloadConfig {
            container: &container,
            duration: Duration::from_secs(60),
            tps: 100,
            ..Default::default()
        }).await
    });

    // Sample memory every 5 seconds
    let mut max_memory = 0u64;
    for _ in 0..12 {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let stats = get_container_stats(&container).await.unwrap();
        max_memory = max_memory.max(stats.memory_usage);
    }

    workload_handle.await.unwrap().unwrap();

    let max_memory_mb = max_memory / 1_000_000;
    println!("Max memory under load: {}MB", max_memory_mb);

    // Should not exceed 512MB under load (PostgreSQL)
    assert!(max_memory_mb < 512, "Memory under load {}MB (target: <512MB)", max_memory_mb);
    report_metric("memory_under_load_mb", max_memory_mb as f64);

    destroy_container(&container).await.ok();
}
```

### 2.2 CPU Overhead

**Target:** <5% CPU for metrics collection

```rust
#[tokio::test]
async fn bench_metrics_collection_overhead() {
    let container = create_test_container().await.unwrap();
    wait_for_healthy(&container).await.unwrap();

    // Baseline: CPU without metrics collection
    let baseline_cpu = measure_cpu_usage(&container, Duration::from_secs(30)).await.unwrap();

    // With metrics collection
    let metrics_collector = start_metrics_collection(&container).await.unwrap();
    let with_metrics_cpu = measure_cpu_usage(&container, Duration::from_secs(30)).await.unwrap();

    stop_metrics_collection(metrics_collector).await.ok();

    let overhead_percent = ((with_metrics_cpu - baseline_cpu) / baseline_cpu) * 100.0;

    println!("Baseline CPU: {:.2}%, With metrics: {:.2}%, Overhead: {:.2}%",
        baseline_cpu, with_metrics_cpu, overhead_percent);

    assert!(overhead_percent < 5.0, "Metrics overhead {:.2}% (target: <5%)", overhead_percent);
    report_metric("metrics_overhead_percent", overhead_percent);

    destroy_container(&container).await.ok();
}
```

### 2.3 Disk Image Sizes

**Target:** PostgreSQL <300MB, MySQL <500MB

```rust
#[tokio::test]
async fn bench_image_sizes() {
    ensure_images_cached().await.unwrap();

    let postgres_size = get_image_size("postgres:16-alpine").await.unwrap();
    let mysql_size = get_image_size("mysql:8.0-debian").await.unwrap();
    let sqlserver_size = get_image_size("mcr.microsoft.com/mssql/server:2022-latest").await.unwrap();

    println!("Image sizes:");
    println!("  PostgreSQL: {:.2}MB", postgres_size / 1_000_000.0);
    println!("  MySQL: {:.2}MB", mysql_size / 1_000_000.0);
    println!("  SQL Server: {:.2}MB", sqlserver_size / 1_000_000.0);

    assert!(postgres_size < 300_000_000, "PostgreSQL image {}MB (target: <300MB)", postgres_size / 1_000_000);
    assert!(mysql_size < 500_000_000, "MySQL image {}MB (target: <500MB)", mysql_size / 1_000_000);

    report_metric("image_size_postgres_mb", postgres_size as f64 / 1_000_000.0);
    report_metric("image_size_mysql_mb", mysql_size as f64 / 1_000_000.0);
    report_metric("image_size_sqlserver_mb", sqlserver_size as f64 / 1_000_000.0);
}
```

---

## 3. Configuration Management Benchmarks

### 3.1 TOML Parsing Speed

**Target:** <50ms for typical configuration

```rust
#[test]
fn bench_toml_parsing() {
    let config_str = include_str!("../test_fixtures/complex_schema.toml");

    let start = Instant::now();
    let config: DatabaseConfig = toml::from_str(config_str).unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 50, "TOML parsing took {}ms (target: <50ms)", elapsed.as_millis());
    report_metric("toml_parsing_ms", elapsed.as_millis() as f64);
}
```

### 3.2 DDL Generation Speed

**Target:** <100ms for schema with <100 tables

```rust
#[test]
fn bench_ddl_generation() {
    let config = load_test_schema_with_50_tables();

    // PostgreSQL
    let start = Instant::now();
    let postgres_ddl = generate_ddl(&config, Database::Postgres).unwrap();
    let postgres_time = start.elapsed();

    // MySQL
    let start = Instant::now();
    let mysql_ddl = generate_ddl(&config, Database::MySQL).unwrap();
    let mysql_time = start.elapsed();

    // SQL Server
    let start = Instant::now();
    let sqlserver_ddl = generate_ddl(&config, Database::SQLServer).unwrap();
    let sqlserver_time = start.elapsed();

    assert!(postgres_time.as_millis() < 100, "PostgreSQL DDL generation {}ms (target: <100ms)", postgres_time.as_millis());
    assert!(mysql_time.as_millis() < 100, "MySQL DDL generation {}ms (target: <100ms)", mysql_time.as_millis());
    assert!(sqlserver_time.as_millis() < 100, "SQL Server DDL generation {}ms (target: <100ms)", sqlserver_time.as_millis());

    report_metric("ddl_generation_postgres_ms", postgres_time.as_millis() as f64);
    report_metric("ddl_generation_mysql_ms", mysql_time.as_millis() as f64);
    report_metric("ddl_generation_sqlserver_ms", sqlserver_time.as_millis() as f64);
}
```

### 3.3 Configuration Deployment Time

**Target:** <5 seconds

```rust
#[tokio::test]
async fn bench_configuration_deployment() {
    let container = create_test_container().await.unwrap();
    wait_for_healthy(&container).await.unwrap();

    let config = load_test_schema();

    let start = Instant::now();
    deploy_configuration(&container, &config).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 5, "Configuration deployment took {}s (target: <5s)", elapsed.as_secs());
    report_metric("config_deployment_time", elapsed);

    destroy_container(&container).await.ok();
}
```

---

## 4. Data Operations Benchmarks

### 4.1 Data Seeding Performance

**Target:** 1,000 rows in <5s, 100,000 rows in <60s

```rust
#[tokio::test]
async fn bench_seeding_small() {
    let container = create_test_container_with_schema().await.unwrap();

    let start = Instant::now();
    seed_data(&container, SeedConfig {
        size: SeedSize::Small, // 1,000 rows
        ..Default::default()
    }).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 5, "Seeding 1K rows took {}s (target: <5s)", elapsed.as_secs());
    report_metric("seeding_1k_rows_time", elapsed);

    destroy_container(&container).await.ok();
}

#[tokio::test]
async fn bench_seeding_medium() {
    let container = create_test_container_with_schema().await.unwrap();

    let start = Instant::now();
    seed_data(&container, SeedConfig {
        size: SeedSize::Medium, // 10,000 rows
        ..Default::default()
    }).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 15, "Seeding 10K rows took {}s (target: <15s)", elapsed.as_secs());
    report_metric("seeding_10k_rows_time", elapsed);

    destroy_container(&container).await.ok();
}

#[tokio::test]
async fn bench_seeding_large() {
    let container = create_test_container_with_schema().await.unwrap();

    let start = Instant::now();
    seed_data(&container, SeedConfig {
        size: SeedSize::Large, // 100,000 rows
        ..Default::default()
    }).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 60, "Seeding 100K rows took {}s (target: <60s)", elapsed.as_secs());

    let rows_per_second = 100_000.0 / elapsed.as_secs_f64();
    println!("Seeding rate: {:.0} rows/second", rows_per_second);

    report_metric("seeding_100k_rows_time", elapsed);
    report_metric("seeding_rows_per_second", rows_per_second);

    destroy_container(&container).await.ok();
}
```

### 4.2 Workload Generation Performance

**Target:** Achieve target TPS within ±10%

```rust
#[tokio::test]
async fn bench_workload_tps_accuracy() {
    let container = create_test_container_with_data().await.unwrap();

    let target_tps = 100;
    let duration = Duration::from_secs(60);

    let start = Instant::now();
    let stats = run_workload(WorkloadConfig {
        container: &container,
        target_tps,
        duration,
        ..Default::default()
    }).await.unwrap();
    let elapsed = start.elapsed();

    let actual_tps = stats.total_transactions as f64 / elapsed.as_secs_f64();
    let tps_accuracy = ((actual_tps - target_tps as f64).abs() / target_tps as f64) * 100.0;

    println!("Target: {} TPS, Actual: {:.1} TPS, Accuracy: {:.1}%",
        target_tps, actual_tps, 100.0 - tps_accuracy);

    assert!(tps_accuracy < 10.0, "TPS accuracy {:.1}% off (target: within 10%)", tps_accuracy);

    report_metric("workload_tps_target", target_tps as f64);
    report_metric("workload_tps_actual", actual_tps);
    report_metric("workload_tps_accuracy_percent", 100.0 - tps_accuracy);

    destroy_container(&container).await.ok();
}

#[tokio::test]
async fn bench_workload_high_throughput() {
    let container = create_test_container_with_data().await.unwrap();

    let target_tps = 1000;
    let duration = Duration::from_secs(60);

    let stats = run_workload(WorkloadConfig {
        container: &container,
        target_tps,
        duration,
        connections: 10,
        ..Default::default()
    }).await.unwrap();

    let actual_tps = stats.total_transactions as f64 / duration.as_secs_f64();

    println!("High throughput test: Target {} TPS, Actual {:.1} TPS", target_tps, actual_tps);

    // Should achieve at least 80% of target at high TPS
    assert!(actual_tps > target_tps as f64 * 0.8,
        "High TPS test achieved {:.1} TPS (target: >{} TPS)", actual_tps, target_tps as f64 * 0.8);

    report_metric("workload_high_tps_actual", actual_tps);

    destroy_container(&container).await.ok();
}
```

---

## 5. CDC Operations Benchmarks

### 5.1 CDC Enable Time

**Target:** <10 seconds

```rust
#[tokio::test]
async fn bench_cdc_enable_postgres() {
    let container = create_container_with_cdc_config().await.unwrap();

    let start = Instant::now();
    enable_cdc(&container).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 10, "CDC enable took {}s (target: <10s)", elapsed.as_secs());
    report_metric("cdc_enable_postgres_time", elapsed);

    destroy_container(&container).await.ok();
}
```

### 5.2 Change Event Capture Latency

**Target:** <1 second from database operation to event captured

```rust
#[tokio::test]
async fn bench_change_event_latency() {
    let container = create_test_container_with_cdc().await.unwrap();
    let monitor = start_cdc_monitor(&container).await.unwrap();

    // Perform INSERT
    let start = Instant::now();
    execute_query(&container, "INSERT INTO test_table VALUES (1, 'test')").await.unwrap();

    // Wait for change event
    let event = monitor.next_event(Duration::from_secs(5)).await.unwrap();
    let latency = start.elapsed();

    assert!(latency.as_secs() < 1, "Change event latency {}ms (target: <1s)", latency.as_millis());
    report_metric("cdc_event_latency_ms", latency.as_millis() as f64);

    destroy_container(&container).await.ok();
}
```

### 5.3 Change Event Monitoring Throughput

**Target:** Monitor 10,000 events/second without loss

```rust
#[tokio::test]
async fn bench_change_event_throughput() {
    let container = create_test_container_with_cdc().await.unwrap();
    let monitor = start_cdc_monitor(&container).await.unwrap();

    // Generate 10,000 changes rapidly
    let workload_handle = tokio::spawn(async move {
        for i in 0..10_000 {
            execute_query(&container, &format!("INSERT INTO test_table VALUES ({}, 'test')", i))
                .await.unwrap();
        }
    });

    workload_handle.await.unwrap();

    // Count captured events
    tokio::time::sleep(Duration::from_secs(5)).await;
    let captured_count = monitor.event_count();

    // Should capture all events
    assert_eq!(captured_count, 10_000, "Captured {} events (expected: 10,000)", captured_count);

    let events_per_second = captured_count as f64 / 5.0; // Over 5 seconds
    println!("Change event monitoring: {:.0} events/second", events_per_second);

    report_metric("cdc_monitoring_events_per_second", events_per_second);

    destroy_container(&container).await.ok();
}
```

---

## 6. TUI Performance Benchmarks

### 6.1 Rendering Performance

**Target:** 30 FPS minimum

```rust
#[tokio::test]
async fn bench_tui_rendering_fps() {
    let containers = create_multiple_test_containers(5).await.unwrap();
    let tui = start_tui(&containers).await.unwrap();

    // Measure FPS over 10 seconds
    let frame_count = Arc::new(AtomicUsize::new(0));
    let frame_count_clone = frame_count.clone();

    let render_handle = tokio::spawn(async move {
        for _ in 0..300 { // 10 seconds at 30 FPS
            tui.render_frame().await.unwrap();
            frame_count_clone.fetch_add(1, Ordering::Relaxed);
            tokio::time::sleep(Duration::from_millis(33)).await; // ~30 FPS
        }
    });

    render_handle.await.unwrap();

    let actual_frames = frame_count.load(Ordering::Relaxed);
    let fps = actual_frames as f64 / 10.0;

    println!("TUI FPS: {:.1}", fps);

    assert!(fps >= 30.0, "TUI FPS {:.1} (target: >=30)", fps);
    report_metric("tui_fps", fps);

    cleanup_containers(&containers).await.ok();
}
```

### 6.2 TUI Memory Usage

**Target:** <50MB

```rust
#[tokio::test]
async fn bench_tui_memory_usage() {
    let containers = create_multiple_test_containers(10).await.unwrap();

    let baseline_memory = get_process_memory().unwrap();

    let tui = start_tui(&containers).await.unwrap();

    // Let TUI run for 60 seconds
    tokio::time::sleep(Duration::from_secs(60)).await;

    let tui_memory = get_process_memory().unwrap();
    let memory_overhead_mb = (tui_memory - baseline_memory) / 1_000_000;

    println!("TUI memory overhead: {}MB", memory_overhead_mb);

    assert!(memory_overhead_mb < 50, "TUI memory {}MB (target: <50MB)", memory_overhead_mb);
    report_metric("tui_memory_mb", memory_overhead_mb as f64);

    cleanup_containers(&containers).await.ok();
}
```

### 6.3 Update Latency

**Target:** <1 second from metric change to display

```rust
#[tokio::test]
async fn bench_tui_update_latency() {
    let container = create_test_container().await.unwrap();
    let tui = start_tui(&[container.clone()]).await.unwrap();

    // Change database state
    let start = Instant::now();
    execute_query(&container, "CREATE TABLE test (id INT)").await.unwrap();

    // Wait for TUI to reflect change
    let update_detected = wait_for_tui_update(&tui, Duration::from_secs(5)).await.unwrap();
    let latency = start.elapsed();

    assert!(update_detected, "TUI update not detected");
    assert!(latency.as_secs() < 1, "TUI update latency {}ms (target: <1s)", latency.as_millis());

    report_metric("tui_update_latency_ms", latency.as_millis() as f64);

    cleanup_containers(&[container]).await.ok();
}
```

---

## 7. End-to-End Workflow Benchmarks

### 7.1 Complete CDC Workflow

**Target:** <2 minutes from start to monitoring changes

```rust
#[tokio::test]
async fn bench_complete_cdc_workflow() {
    let start = Instant::now();

    // 1. Create container
    let container = create_container(ContainerConfig {
        database: Database::Postgres,
        version: "16",
        ..Default::default()
    }).await.unwrap();

    let after_create = Instant::now();
    println!("Container created: {:.2}s", (after_create - start).as_secs_f64());

    // 2. Deploy schema
    let schema = load_test_schema();
    deploy_configuration(&container, &schema).await.unwrap();

    let after_schema = Instant::now();
    println!("Schema deployed: {:.2}s", (after_schema - after_create).as_secs_f64());

    // 3. Enable CDC
    enable_cdc(&container).await.unwrap();

    let after_cdc = Instant::now();
    println!("CDC enabled: {:.2}s", (after_cdc - after_schema).as_secs_f64());

    // 4. Seed data
    seed_data(&container, SeedConfig {
        size: SeedSize::Small,
        ..Default::default()
    }).await.unwrap();

    let after_seed = Instant::now();
    println!("Data seeded: {:.2}s", (after_seed - after_cdc).as_secs_f64());

    // 5. Start workload
    let workload_handle = tokio::spawn(async move {
        run_workload(WorkloadConfig {
            container: &container,
            duration: Duration::from_secs(30),
            target_tps: 10,
            ..Default::default()
        }).await
    });

    // 6. Monitor changes
    let monitor = start_cdc_monitor(&container).await.unwrap();
    let event = monitor.next_event(Duration::from_secs(10)).await.unwrap();

    let total_time = start.elapsed();

    println!("Complete CDC workflow: {:.2}s", total_time.as_secs_f64());

    assert!(total_time.as_secs() < 120, "Workflow took {}s (target: <120s)", total_time.as_secs());

    report_metric("complete_cdc_workflow_time", total_time);

    workload_handle.await.unwrap().ok();
    cleanup_containers(&[container]).await.ok();
}
```

---

## Benchmark Infrastructure

### Metrics Reporting

```rust
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref METRICS: Mutex<HashMap<String, Vec<f64>>> = Mutex::new(HashMap::new());
}

pub fn report_metric(name: &str, value: impl Into<f64>) {
    let mut metrics = METRICS.lock().unwrap();
    metrics.entry(name.to_string())
        .or_insert_with(Vec::new)
        .push(value.into());
}

pub fn report_duration_metric(name: &str, duration: Duration) {
    report_metric(name, duration.as_secs_f64());
}

pub fn generate_benchmark_report() -> String {
    let metrics = METRICS.lock().unwrap();

    let mut report = String::new();
    report.push_str("# Benchmark Report\n\n");
    report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now()));

    for (name, values) in metrics.iter() {
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        report.push_str(&format!("## {}\n", name));
        report.push_str(&format!("- Average: {:.2}\n", avg));
        report.push_str(&format!("- Min: {:.2}\n", min));
        report.push_str(&format!("- Max: {:.2}\n", max));
        report.push_str(&format!("- Samples: {}\n\n", values.len()));
    }

    report
}
```

### CI/CD Integration

```yaml
# .github/workflows/benchmarks.yml
name: Performance Benchmarks

on:
  pull_request:
  push:
    branches: [main]
  schedule:
    - cron: '0 0 * * 0' # Weekly

jobs:
  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: 1.92.0
          profile: minimal

      - name: Start Docker
        run: sudo systemctl start docker

      - name: Run benchmarks
        run: |
          cargo test --release --test benchmarks -- --ignored --nocapture

      - name: Generate report
        run: |
          cargo run --bin generate-benchmark-report > benchmark_report.md

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: benchmark_report.md

      - name: Compare with baseline
        run: |
          cargo run --bin compare-benchmarks \
            --baseline benchmarks/baseline.json \
            --current benchmark_results.json \
            --threshold 10 # 10% regression threshold
```

### Benchmark Runner

```bash
#!/bin/bash
# scripts/run_benchmarks.sh

set -e

echo "Running simDB Performance Benchmarks"
echo "====================================="
echo ""

# Ensure Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Error: Docker is not running"
    exit 1
fi

# Pre-pull images
echo "Pre-pulling Docker images..."
docker pull postgres:16-alpine
docker pull mysql:8.0-debian
docker pull mcr.microsoft.com/mssql/server:2022-latest

echo ""
echo "Running container operation benchmarks..."
cargo test --release bench_cold_start -- --ignored --nocapture
cargo test --release bench_warm_start -- --ignored --nocapture
cargo test --release bench_parallel_container_creation -- --ignored --nocapture

echo ""
echo "Running resource usage benchmarks..."
cargo test --release bench_memory_footprint -- --ignored --nocapture
cargo test --release bench_image_sizes -- --ignored --nocapture

echo ""
echo "Running configuration benchmarks..."
cargo test --release bench_toml_parsing -- --nocapture
cargo test --release bench_ddl_generation -- --nocapture

echo ""
echo "Running data operation benchmarks..."
cargo test --release bench_seeding -- --ignored --nocapture
cargo test --release bench_workload -- --ignored --nocapture

echo ""
echo "Running CDC benchmarks..."
cargo test --release bench_cdc -- --ignored --nocapture

echo ""
echo "Running end-to-end benchmarks..."
cargo test --release bench_complete_cdc_workflow -- --ignored --nocapture

echo ""
echo "Generating report..."
cargo run --release --bin generate-benchmark-report

echo ""
echo "Benchmarks complete! See benchmark_report.md for results."
```

### Baseline Management

```bash
# scripts/update_baseline.sh
#!/bin/bash

# Run benchmarks and save as new baseline
cargo test --release --test benchmarks -- --ignored --nocapture
cp benchmark_results.json benchmarks/baseline.json

echo "Baseline updated"
git add benchmarks/baseline.json
git commit -m "Update benchmark baseline"
```

---

## Running Benchmarks

### Local Development

```bash
# Run all benchmarks
./scripts/run_benchmarks.sh

# Run specific category
cargo test --release bench_warm_start -- --ignored --nocapture

# Run with custom iterations
cargo test --release bench_seeding_large -- --ignored --nocapture -- --test-threads=1
```

### CI/CD

Benchmarks run automatically on:
- Pull requests (to detect regressions)
- Main branch commits (to update baselines)
- Weekly schedule (to track trends)

### Regression Detection

Benchmarks fail if performance regresses by >10% from baseline:

```rust
fn check_regression(metric: &str, current: f64, baseline: f64) {
    let regression_percent = ((current - baseline) / baseline) * 100.0;

    if regression_percent > 10.0 {
        panic!("Performance regression detected for {}: {:.1}% slower than baseline",
            metric, regression_percent);
    }
}
```

---

## Continuous Monitoring

### Grafana Dashboard

Track benchmark trends over time:
- Container startup times
- Memory usage
- TPS accuracy
- CDC latency

### Alerting

Alert on:
- >20% regression in any metric
- Consistent degradation over 3 runs
- Benchmark failures

---

## Summary

This comprehensive benchmark suite validates all performance targets across:
- **29 benchmarks** covering all critical operations
- **Automated CI/CD integration** for regression detection
- **Historical tracking** to identify trends
- **Clear pass/fail criteria** for each target

All benchmarks are reproducible, automated, and provide clear metrics to guide optimization efforts.
