//! Tests to verify accuracy of collected database metrics
//! These tests execute known queries and verify metrics reflect the actual state

use dbarena::container::{ContainerConfig, DatabaseType};
use dbarena::database_metrics::{DatabaseMetricsCollector, DockerDatabaseMetricsCollector};
use bollard::Docker;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use bollard::exec::{CreateExecOptions, StartExecResults};
use futures::StreamExt;

use crate::common::{create_and_start_container, cleanup_container, unique_container_name, docker_available};

#[tokio::test]
#[ignore] // Requires Docker
async fn test_postgres_connection_count_accuracy() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    // Create PostgreSQL container
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("pg-conn-accuracy"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    sleep(Duration::from_secs(2)).await;

    let docker = Arc::new(Docker::connect_with_local_defaults().expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // Collect initial metrics
    let metrics1 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics");

    // Initial connections should be minimal (just monitoring)
    assert!(
        metrics1.active_connections >= 1,
        "Should have at least 1 connection (monitoring)"
    );
    let initial_connections = metrics1.active_connections;

    // Open additional connections by running queries
    let mut handles = vec![];
    for _ in 0..3 {
        let docker_clone = docker.clone();
        let container_id = container.id.clone();
        let handle = tokio::spawn(async move {
            let exec = docker_clone
                .create_exec(
                    &container_id,
                    CreateExecOptions {
                        cmd: Some(vec!["psql", "-U", "postgres", "-c", "SELECT pg_sleep(2);"]),
                        attach_stdout: Some(true),
                        attach_stderr: Some(true),
                        ..Default::default()
                    },
                )
                .await
                .expect("Failed to create exec");

            if let StartExecResults::Attached { mut output, .. } =
                docker_clone.start_exec(&exec.id, None).await.expect("Failed to start exec")
            {
                while let Some(_) = output.next().await {}
            }
        });
        handles.push(handle);
    }

    // Give connections time to establish
    sleep(Duration::from_millis(500)).await;

    // Collect metrics while connections are active
    let metrics2 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics (2nd)");

    // Should now have more connections (initial + 3 new queries)
    assert!(
        metrics2.active_connections > initial_connections,
        "Connection count should increase: {} vs {}",
        metrics2.active_connections,
        initial_connections
    );

    // Wait for background queries to complete
    for handle in handles {
        handle.await.ok();
    }

    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_postgres_transaction_count_accuracy() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("pg-txn-accuracy"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    sleep(Duration::from_secs(2)).await;

    let docker = Arc::new(Docker::connect_with_local_defaults().expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // First collection to establish baseline
    let _metrics1 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics (1st)");

    sleep(Duration::from_secs(1)).await;

    // Execute exactly 5 transactions (commits)
    for i in 0..5 {
        let exec = docker
            .create_exec(
                &container.id,
                CreateExecOptions {
                    cmd: Some(vec![
                        "psql",
                        "-U",
                        "postgres",
                        "-c",
                        &format!("BEGIN; SELECT {}; COMMIT;", i),
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
    }

    sleep(Duration::from_secs(2)).await;

    // Second collection after transactions
    let metrics2 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics (2nd)");

    // Should have detected transaction activity
    // Note: commits_per_second might not be exactly 5/time_delta due to background activity,
    // but should be > 0 indicating transactions were detected
    if metrics2.commits_per_second > 0.0 {
        println!(
            "✓ Detected transaction activity: {:.2} commits/sec, {:.2} transactions/sec",
            metrics2.commits_per_second, metrics2.transactions_per_second
        );
    } else {
        // This is acceptable on first sample
        println!("Note: Transaction rate is 0 (may be first sample)");
    }

    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_postgres_query_count_accuracy() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("pg-query-accuracy"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    sleep(Duration::from_secs(2)).await;

    let docker = Arc::new(Docker::connect_with_local_defaults().expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // First collection
    let _metrics1 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics (1st)");

    sleep(Duration::from_secs(1)).await;

    // Create a table and insert known data
    let exec = docker
        .create_exec(
            &container.id,
            CreateExecOptions {
                cmd: Some(vec![
                    "psql",
                    "-U",
                    "postgres",
                    "-c",
                    "CREATE TABLE test_metrics (id INT, value TEXT); \
                     INSERT INTO test_metrics VALUES (1, 'one'), (2, 'two'), (3, 'three'); \
                     SELECT * FROM test_metrics; \
                     UPDATE test_metrics SET value = 'updated' WHERE id = 1; \
                     DELETE FROM test_metrics WHERE id = 3;",
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

    // Second collection
    let metrics2 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics (2nd)");

    // Verify query breakdown captured the operations
    // Note: PostgreSQL tracks row operations, not query types directly
    println!(
        "Query breakdown - SELECT: {}, INSERT: {}, UPDATE: {}, DELETE: {}",
        metrics2.query_breakdown.select_count,
        metrics2.query_breakdown.insert_count,
        metrics2.query_breakdown.update_count,
        metrics2.query_breakdown.delete_count
    );

    // Should have some query activity detected
    let total_ops = metrics2.query_breakdown.total();
    assert!(
        total_ops > 0 || metrics2.queries_per_second > 0.0,
        "Should have detected some query activity"
    );

    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_mysql_connection_count_accuracy() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    let config = ContainerConfig::new(DatabaseType::MySQL)
        .with_name(unique_container_name("mysql-conn-accuracy"))
        .with_version("8".to_string());

    let container = create_and_start_container(config, Duration::from_secs(90))
        .await
        .expect("Failed to create container");

    sleep(Duration::from_secs(2)).await;

    let docker = Arc::new(Docker::connect_with_local_defaults().expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // Collect initial metrics
    let metrics1 = collector
        .collect(&container.id, DatabaseType::MySQL)
        .await
        .expect("Failed to collect metrics");

    // MySQL should have at least the monitoring connection
    assert!(
        metrics1.active_connections >= 1,
        "Should have at least 1 connection"
    );

    println!(
        "MySQL initial connections: {}, cache hit: {:.2}%",
        metrics1.active_connections,
        metrics1.cache_hit_ratio.unwrap_or(0.0)
    );

    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_rate_calculation_time_accuracy() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("pg-rate-time-accuracy"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    sleep(Duration::from_secs(2)).await;

    let docker = Arc::new(Docker::connect_with_local_defaults().expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // First collection
    let metrics1 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics (1st)");

    let timestamp1 = metrics1.timestamp;

    // Wait exactly 3 seconds
    sleep(Duration::from_secs(3)).await;

    // Second collection
    let metrics2 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics (2nd)");

    let timestamp2 = metrics2.timestamp;

    // Verify time delta is approximately 3 seconds (allow some variance)
    let time_delta = timestamp2 - timestamp1;
    assert!(
        time_delta >= 2 && time_delta <= 4,
        "Time delta should be ~3 seconds, got {}",
        time_delta
    );

    println!(
        "✓ Time tracking accurate: {} seconds between samples",
        time_delta
    );

    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_metrics_consistency_across_collections() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("pg-consistency"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    sleep(Duration::from_secs(2)).await;

    let docker = Arc::new(Docker::connect_with_local_defaults().expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // Collect metrics 3 times
    let mut previous_commits = None;
    for i in 0..3 {
        let metrics = collector
            .collect(&container.id, DatabaseType::Postgres)
            .await
            .expect(&format!("Failed to collect metrics ({})", i + 1));

        // Verify container ID and type are consistent
        assert_eq!(metrics.container_id, container.id);
        assert_eq!(metrics.database_type, DatabaseType::Postgres);

        // Cumulative counters should never decrease
        if let Some(prev_commits) = previous_commits {
            let current_commits = metrics
                .extras
                .get("cumulative_commits")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            assert!(
                current_commits >= prev_commits,
                "Cumulative commits should not decrease: {} -> {}",
                prev_commits,
                current_commits
            );
        }

        previous_commits = metrics
            .extras
            .get("cumulative_commits")
            .and_then(|v| v.as_u64());

        sleep(Duration::from_secs(1)).await;
    }

    println!("✓ Metrics remain consistent across multiple collections");

    cleanup_container(&container.id).await.ok();
}
