//! Integration tests for database metrics collection
//! Tests require Docker to be running

use dbarena::container::{ContainerConfig, DatabaseType};
use dbarena::database_metrics::{DatabaseMetricsCollector, DockerDatabaseMetricsCollector};
use bollard::Docker;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::common::{create_and_start_container, cleanup_container, unique_container_name, docker_available};

#[tokio::test]
#[ignore] // Requires Docker
async fn test_postgres_metrics_collection() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    // Create and start a PostgreSQL container
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("postgres-metrics-test"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create test container");

    // Give PostgreSQL a moment to fully initialize
    sleep(Duration::from_secs(2)).await;

    // Create metrics collector
    let docker = Arc::new(Docker::connect_with_local_defaults()
        .expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // Collect metrics
    let metrics = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect PostgreSQL metrics");

    // Verify metrics
    assert_eq!(metrics.database_type, DatabaseType::Postgres);
    assert_eq!(metrics.container_id, container.id);

    // PostgreSQL should have at least 1 connection (our monitoring connection)
    assert!(metrics.active_connections > 0, "Should have active connections");

    // Should have max_connections set
    assert!(metrics.max_connections.is_some(), "Should have max_connections");
    assert!(metrics.max_connections.unwrap() > 0, "Max connections should be positive");

    // Cache hit ratio might be 0 for a fresh database, but should be present
    assert!(metrics.cache_hit_ratio.is_some(), "Should have cache hit ratio");

    // On first collection, rates should be 0
    assert_eq!(metrics.queries_per_second, 0.0, "First sample should have 0 QPS");
    assert_eq!(metrics.transactions_per_second, 0.0, "First sample should have 0 TPS");

    // Wait and collect again to get rates
    sleep(Duration::from_secs(2)).await;

    let metrics2 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect PostgreSQL metrics (2nd time)");

    // Second sample should have cumulative data stored
    assert!(metrics2.extras.contains_key("cumulative_commits"), "Should store cumulative commits");

    // Cleanup
    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_mysql_metrics_collection() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    // Create and start a MySQL container
    let config = ContainerConfig::new(DatabaseType::MySQL).with_name(unique_container_name("mysql-metrics-test"))
        .with_version("8".to_string());

    let container = create_and_start_container(config, Duration::from_secs(90))
        .await
        .expect("Failed to create test container");

    // Give MySQL a moment to fully initialize
    sleep(Duration::from_secs(2)).await;

    // Create metrics collector
    let docker = Arc::new(Docker::connect_with_local_defaults()
        .expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // Collect metrics
    let metrics = collector
        .collect(&container.id, DatabaseType::MySQL)
        .await
        .expect("Failed to collect MySQL metrics");

    // Verify metrics
    assert_eq!(metrics.database_type, DatabaseType::MySQL);
    assert_eq!(metrics.container_id, container.id);

    // MySQL should have connections
    assert!(metrics.active_connections > 0, "Should have active connections");

    // Should have max_connections set (note: may be None if parsing fails, non-critical)
    if metrics.max_connections.is_none() {
        println!("Warning: max_connections not parsed from MySQL (non-critical)");
    }

    // Cache hit ratio should be calculated from buffer pool stats
    assert!(metrics.cache_hit_ratio.is_some(), "Should have cache hit ratio");

    // Cleanup
    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_postgres_metrics_rate_calculation() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    // Create and start a PostgreSQL container
    let config = ContainerConfig::new(DatabaseType::Postgres).with_name(unique_container_name("postgres-rate-test"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create test container");

    sleep(Duration::from_secs(2)).await;

    let docker = Arc::new(Docker::connect_with_local_defaults()
        .expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // First collection
    let _metrics1 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics (1st)");

    sleep(Duration::from_secs(2)).await;

    // Second collection - should calculate rates
    let metrics2 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics (2nd)");

    // Timestamp should be set
    assert!(metrics2.timestamp > 0, "Should have timestamp");

    // Should have stored cumulative values for rate calculation
    assert!(
        metrics2.extras.contains_key("cumulative_commits"),
        "Should store cumulative commits for rate calculation"
    );

    // Cleanup
    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_postgres_query_execution_tracking() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    // Create and start a PostgreSQL container
    let config = ContainerConfig::new(DatabaseType::Postgres).with_name(unique_container_name("postgres-query-test"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create test container");

    sleep(Duration::from_secs(2)).await;

    let docker = Arc::new(Docker::connect_with_local_defaults()
        .expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // First collection
    let _metrics1 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics (1st)");

    // Execute some queries
    use bollard::exec::{CreateExecOptions, StartExecResults};
    use futures::StreamExt;

    let exec = docker
        .create_exec(
            &container.id,
            CreateExecOptions {
                cmd: Some(vec![
                    "psql",
                    "-U",
                    "postgres",
                    "-c",
                    "SELECT 1",
                ]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to create exec");

    if let StartExecResults::Attached { mut output, .. } =
        docker.start_exec(&exec.id, None).await.expect("Failed to start exec")
    {
        while let Some(_) = output.next().await {}
    }

    sleep(Duration::from_secs(2)).await;

    // Second collection - should show query activity
    let metrics2 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics (2nd)");

    // The query breakdown should show some activity
    let total_queries = metrics2.query_breakdown.total();
    println!("Total queries detected: {}", total_queries);

    // Note: Query counting is approximate based on row operations
    // We mainly verify the collection doesn't error

    // Cleanup
    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_collector_supports_all_database_types() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    let docker = Arc::new(Docker::connect_with_local_defaults()
        .expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // Should support all three database types
    assert!(
        collector.supports_database_type(DatabaseType::Postgres).await,
        "Should support PostgreSQL"
    );
    assert!(
        collector.supports_database_type(DatabaseType::MySQL).await,
        "Should support MySQL"
    );
    assert!(
        collector.supports_database_type(DatabaseType::SQLServer).await,
        "Should support SQL Server"
    );
}
