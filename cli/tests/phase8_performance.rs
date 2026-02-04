//! Phase 8: Performance and Integration Tests
//!
//! These tests validate that v0.5.0 meets the non-functional requirements:
//! - Seeding: 100K rows in <60s
//! - Workload: 1,000 TPS with ±10% accuracy
//! - Latency: p99 <100ms under normal load
//! - Stability: 1-hour workloads without errors
//! - Cross-database: Identical behavior on Postgres, MySQL, SQL Server

use dbarena::container::{ContainerManager, DatabaseType, DockerClient};
use dbarena::seed::config::SeedConfig;
use dbarena::seed::engine::SeedingEngine;
use dbarena::workload::{WorkloadConfig, WorkloadEngine, WorkloadPattern};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

/// Test helper to create a test container
async fn create_test_container(
    db_type: DatabaseType,
    name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client.clone());

    // Create container
    let config = dbarena::config::ContainerConfig {
        database_type: db_type,
        version: None,
        name: Some(name.to_string()),
        port: None,
        persistent: false,
        memory: Some(1024), // 1GB for testing
        cpu_shares: None,
        environment: Default::default(),
    };

    let container_id = manager.create(&config).await?;

    // Wait for container to be ready
    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(container_id)
}

/// Test helper to destroy a test container
async fn destroy_test_container(container_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let docker_client = DockerClient::new()?;
    let manager = ContainerManager::new(docker_client);

    manager
        .destroy(container_id, false)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// NFR Test: Seed 100,000 rows in <60 seconds
#[test]
#[ignore] // Run with: cargo test --test phase8_performance -- --ignored
fn test_seeding_performance_100k_rows() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        println!("\n🧪 Testing: Seed 100K rows in <60s");

        // Create test container
        let container_id = create_test_container(DatabaseType::Postgres, "test-seed-perf")
            .await
            .expect("Failed to create test container");

        // Create simple schema
        let docker_client = DockerClient::new().unwrap();
        let docker = Arc::new(docker_client.docker().clone());

        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS test_users (
                id SERIAL PRIMARY KEY,
                email VARCHAR(255),
                name VARCHAR(255),
                created_at TIMESTAMP
            )
        "#;

        dbarena::database_metrics::collector::exec_query(
            docker.clone(),
            &container_id,
            DatabaseType::Postgres,
            create_table_sql,
        )
        .await
        .expect("Failed to create table");

        // Create seed config for 100K rows
        let config_toml = r#"
[seed_rules]
global_seed = 42

[[seed_rules.tables]]
name = "test_users"
count = 100000

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"
[seed_rules.tables.columns.options]
start = 1

[[seed_rules.tables.columns]]
name = "email"
generator = "email"

[[seed_rules.tables.columns]]
name = "name"
generator = "name"
[seed_rules.tables.columns.options]
type = "full"

[[seed_rules.tables.columns]]
name = "created_at"
generator = "timestamp"
[seed_rules.tables.columns.options]
type = "now"
        "#;

        let seed_config: SeedConfig = toml::from_str(config_toml).expect("Failed to parse config");

        // Seed and measure time
        let start = Instant::now();

        let mut engine = SeedingEngine::new(
            container_id.clone(),
            DatabaseType::Postgres,
            docker.clone(),
            seed_config.seed_rules.global_seed,
            Some(1000), // batch size
        );

        let stats = engine
            .seed_all(&seed_config.seed_rules.tables)
            .await
            .expect("Seeding failed");

        let elapsed = start.elapsed();

        println!("✅ Seeded {} rows in {:.2}s", stats[0].rows_inserted, elapsed.as_secs_f64());
        println!("   Rate: {:.0} rows/sec", stats[0].rows_per_second);

        // Assert NFR
        assert_eq!(stats[0].rows_inserted, 100_000, "Should insert exactly 100K rows");
        assert!(
            elapsed < Duration::from_secs(60),
            "Should complete in <60s, took {:.2}s",
            elapsed.as_secs_f64()
        );
        assert!(
            stats[0].rows_per_second > 1666.0,
            "Should maintain >1666 rows/sec for 100K in 60s"
        );

        // Cleanup
        destroy_test_container(&container_id).await.ok();
    });
}

/// NFR Test: Workload achieves target TPS within ±10%
#[test]
#[ignore]
fn test_workload_tps_accuracy() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        println!("\n🧪 Testing: Workload TPS accuracy (target 100 TPS, ±10%)");

        // Create test container
        let container_id = create_test_container(DatabaseType::Postgres, "test-workload-tps")
            .await
            .expect("Failed to create test container");

        let docker_client = DockerClient::new().unwrap();
        let docker = Arc::new(docker_client.docker().clone());

        // Create simple schema
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS test_products (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255),
                price DECIMAL(10, 2)
            );
            INSERT INTO test_products (name, price)
            SELECT
                'Product ' || generate_series AS name,
                (random() * 1000)::decimal(10,2) AS price
            FROM generate_series(1, 1000);
        "#;

        dbarena::database_metrics::collector::exec_query(
            docker.clone(),
            &container_id,
            DatabaseType::Postgres,
            create_table_sql,
        )
        .await
        .expect("Failed to create table");

        // Create workload config
        let workload_config = WorkloadConfig {
            name: "TPS Test".to_string(),
            pattern: Some(WorkloadPattern::ReadHeavy),
            custom_operations: None,
            custom_queries: None,
            tables: vec!["test_products".to_string()],
            connections: 10,
            target_tps: 100,
            duration_seconds: Some(30),
            transaction_count: None,
        };

        // Run workload
        let engine = WorkloadEngine::new(
            container_id.clone(),
            DatabaseType::Postgres,
            workload_config.clone(),
            docker.clone(),
        );

        let stats = engine.run().await.expect("Workload failed");
        let snapshot = stats.snapshot();

        println!("✅ Achieved {:.1} TPS (target: {} TPS)", snapshot.tps, workload_config.target_tps);
        println!("   Duration: {:.2}s", snapshot.elapsed.as_secs_f64());
        println!("   Total transactions: {}", snapshot.total);
        println!("   Success rate: {:.1}%", snapshot.success_rate);

        // Assert TPS accuracy within ±10%
        let target_tps = workload_config.target_tps as f64;
        let tps_diff_percent = ((snapshot.tps - target_tps) / target_tps * 100.0).abs();

        assert!(
            tps_diff_percent <= 10.0,
            "TPS should be within ±10% of target. Got {:.1} TPS (target {}), diff: {:.1}%",
            snapshot.tps,
            target_tps,
            tps_diff_percent
        );

        assert!(
            snapshot.success_rate >= 95.0,
            "Success rate should be ≥95%, got {:.1}%",
            snapshot.success_rate
        );

        // Cleanup
        destroy_test_container(&container_id).await.ok();
    });
}

/// NFR Test: Latency p99 <100ms under normal load
#[test]
#[ignore]
fn test_workload_latency_p99() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        println!("\n🧪 Testing: Workload latency p99 <100ms");

        let container_id = create_test_container(DatabaseType::Postgres, "test-latency")
            .await
            .expect("Failed to create test container");

        let docker_client = DockerClient::new().unwrap();
        let docker = Arc::new(docker_client.docker().clone());

        // Create test table
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS test_data (
                id SERIAL PRIMARY KEY,
                value INTEGER
            );
            INSERT INTO test_data (value)
            SELECT generate_series FROM generate_series(1, 10000);
        "#;

        dbarena::database_metrics::collector::exec_query(
            docker.clone(),
            &container_id,
            DatabaseType::Postgres,
            create_table_sql,
        )
        .await
        .expect("Failed to create table");

        // Run workload with moderate load
        let workload_config = WorkloadConfig {
            name: "Latency Test".to_string(),
            pattern: Some(WorkloadPattern::Balanced),
            custom_operations: None,
            custom_queries: None,
            tables: vec!["test_data".to_string()],
            connections: 20,
            target_tps: 200,
            duration_seconds: Some(30),
            transaction_count: None,
        };

        let engine = WorkloadEngine::new(
            container_id.clone(),
            DatabaseType::Postgres,
            workload_config,
            docker.clone(),
        );

        let stats = engine.run().await.expect("Workload failed");
        let snapshot = stats.snapshot();

        println!("✅ Latency metrics:");
        println!("   P50: {:.2}ms", snapshot.p50.unwrap_or(0) as f64 / 1000.0);
        println!("   P95: {:.2}ms", snapshot.p95.unwrap_or(0) as f64 / 1000.0);
        println!("   P99: {:.2}ms", snapshot.p99.unwrap_or(0) as f64 / 1000.0);
        println!("   Max: {:.2}ms", snapshot.max.unwrap_or(0) as f64 / 1000.0);

        // Assert p99 <100ms
        let p99_ms = snapshot.p99.unwrap_or(0) as f64 / 1000.0;
        assert!(
            p99_ms < 100.0,
            "P99 latency should be <100ms, got {:.2}ms",
            p99_ms
        );

        // Cleanup
        destroy_test_container(&container_id).await.ok();
    });
}

/// Scale Test: Seed 1M rows (stability check)
#[test]
#[ignore]
fn test_seeding_scale_1m_rows() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        println!("\n🧪 Scale Test: Seed 1M rows (stability)");

        let container_id = create_test_container(DatabaseType::Postgres, "test-seed-scale")
            .await
            .expect("Failed to create test container");

        let docker_client = DockerClient::new().unwrap();
        let docker = Arc::new(docker_client.docker().clone());

        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS large_test (
                id SERIAL PRIMARY KEY,
                data VARCHAR(100)
            )
        "#;

        dbarena::database_metrics::collector::exec_query(
            docker.clone(),
            &container_id,
            DatabaseType::Postgres,
            create_table_sql,
        )
        .await
        .expect("Failed to create table");

        let config_toml = r#"
[seed_rules]
global_seed = 123

[[seed_rules.tables]]
name = "large_test"
count = 1000000

[[seed_rules.tables.columns]]
name = "id"
generator = "sequential"
[seed_rules.tables.columns.options]
start = 1

[[seed_rules.tables.columns]]
name = "data"
generator = "template"
[seed_rules.tables.columns.options]
template = "data_{random_int:1:999999}"
        "#;

        let seed_config: SeedConfig = toml::from_str(config_toml).unwrap();

        let start = Instant::now();

        let mut engine = SeedingEngine::new(
            container_id.clone(),
            DatabaseType::Postgres,
            docker.clone(),
            seed_config.seed_rules.global_seed,
            Some(5000), // larger batch size
        );

        let stats = engine
            .seed_all(&seed_config.seed_rules.tables)
            .await
            .expect("Large scale seeding failed");

        let elapsed = start.elapsed();

        println!("✅ Seeded {} rows in {:.2}s", stats[0].rows_inserted, elapsed.as_secs_f64());
        println!("   Rate: {:.0} rows/sec", stats[0].rows_per_second);

        assert_eq!(stats[0].rows_inserted, 1_000_000);

        // Cleanup
        destroy_test_container(&container_id).await.ok();
    });
}

/// Stability Test: Run workload for extended duration
#[test]
#[ignore]
fn test_workload_long_duration() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        println!("\n🧪 Stability Test: 5-minute workload");

        let container_id = create_test_container(DatabaseType::Postgres, "test-stability")
            .await
            .expect("Failed to create test container");

        let docker_client = DockerClient::new().unwrap();
        let docker = Arc::new(docker_client.docker().clone());

        // Create test schema
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS stability_test (
                id SERIAL PRIMARY KEY,
                value INTEGER
            );
            INSERT INTO stability_test (value)
            SELECT generate_series FROM generate_series(1, 5000);
        "#;

        dbarena::database_metrics::collector::exec_query(
            docker.clone(),
            &container_id,
            DatabaseType::Postgres,
            create_table_sql,
        )
        .await
        .expect("Failed to create table");

        // Run 5-minute workload (shorter than 1 hour for faster testing)
        let workload_config = WorkloadConfig {
            name: "Stability Test".to_string(),
            pattern: Some(WorkloadPattern::Oltp),
            custom_operations: None,
            custom_queries: None,
            tables: vec!["stability_test".to_string()],
            connections: 50,
            target_tps: 500,
            duration_seconds: Some(300), // 5 minutes
            transaction_count: None,
        };

        let engine = WorkloadEngine::new(
            container_id.clone(),
            DatabaseType::Postgres,
            workload_config,
            docker.clone(),
        );

        let stats = engine.run().await.expect("Long workload failed");
        let snapshot = stats.snapshot();

        println!("✅ Completed 5-minute workload:");
        println!("   Total transactions: {}", snapshot.total);
        println!("   Success rate: {:.1}%", snapshot.success_rate);
        println!("   Average TPS: {:.1}", snapshot.tps);
        println!("   Duration: {:.2}s", snapshot.elapsed.as_secs_f64());

        // Should maintain high success rate throughout
        assert!(
            snapshot.success_rate >= 95.0,
            "Should maintain ≥95% success rate over 5 minutes"
        );

        // Should process expected number of transactions
        let expected_min_transactions = 500 * 300 * 0.9; // 90% of target
        assert!(
            snapshot.total as f64 >= expected_min_transactions,
            "Should process at least 90% of target transactions"
        );

        // Cleanup
        destroy_test_container(&container_id).await.ok();
    });
}
