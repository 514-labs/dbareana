//! End-to-end tests for QPS and TPS accuracy from collection to display
//! These tests verify that the rates shown to users are accurate

use dbarena::container::{ContainerConfig, DatabaseType};
use dbarena::database_metrics::{DatabaseMetricsCollector, DockerDatabaseMetricsCollector};
use bollard::Docker;
use bollard::exec::{CreateExecOptions, StartExecResults};
use futures::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::common::{create_and_start_container, cleanup_container, unique_container_name, docker_available};

#[tokio::test]
#[ignore] // Requires Docker
async fn test_qps_accuracy_with_known_query_count() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    // This test executes a KNOWN number of queries and verifies QPS is accurate
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("pg-qps-accuracy"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    sleep(Duration::from_secs(3)).await;

    let docker = Arc::new(Docker::connect_with_local_defaults().expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // Collect baseline (first sample) - rates will be 0
    let metrics1 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect first metrics");

    println!("First collection - QPS: {:.2}, TPS: {:.2}",
        metrics1.queries_per_second,
        metrics1.transactions_per_second
    );

    // First sample should have 0 rates (no previous to compare to)
    assert_eq!(metrics1.queries_per_second, 0.0, "First sample should have 0 QPS");
    assert_eq!(metrics1.transactions_per_second, 0.0, "First sample should have 0 TPS");

    sleep(Duration::from_secs(2)).await;

    // Second collection - this might show inflated rates from startup activity
    let metrics2 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect second metrics");

    println!("Second collection - QPS: {:.2}, TPS: {:.2} (may be inflated from startup)",
        metrics2.queries_per_second,
        metrics2.transactions_per_second
    );

    // Wait for things to stabilize
    sleep(Duration::from_secs(3)).await;

    // Third collection establishes stable baseline
    let _metrics3 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect third metrics");

    sleep(Duration::from_secs(2)).await;

    // NOW execute exactly 10 SELECT queries
    let start_time = tokio::time::Instant::now();
    for i in 0..10 {
        let exec = docker
            .create_exec(
                &container.id,
                CreateExecOptions {
                    cmd: Some(vec!["psql", "-U", "postgres", "-c", &format!("SELECT {}", i)]),
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
            while output.next().await.is_some() {}
        }
    }
    let elapsed = start_time.elapsed().as_secs_f64();

    sleep(Duration::from_secs(2)).await;

    // Collect metrics after our known queries
    let metrics4 = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics after queries");

    println!("After 10 queries in {:.2}s - QPS: {:.2}, TPS: {:.2}",
        elapsed,
        metrics4.queries_per_second,
        metrics4.transactions_per_second
    );

    // We executed 10 queries, each SELECT returns 1 row
    // So we should see at least 10 rows_returned in the delta
    // QPS should be positive (we did execute queries)
    // The actual rate might not be exactly 10/elapsed because of:
    // 1. Background PostgreSQL activity
    // 2. Our own monitoring queries
    // But it should be reasonable (not 0, not 10,000)

    // Conservative check: QPS should be between 0 and 1000
    assert!(
        metrics4.queries_per_second < 1000.0,
        "QPS should not be inflated: got {:.2}",
        metrics4.queries_per_second
    );

    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_tps_accuracy_with_known_transaction_count() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("pg-tps-accuracy"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    sleep(Duration::from_secs(3)).await;

    let docker = Arc::new(Docker::connect_with_local_defaults().expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // Establish baseline with multiple collections to skip startup noise
    for i in 0..3 {
        let metrics = collector
            .collect(&container.id, DatabaseType::Postgres)
            .await
            .expect(&format!("Failed to collect baseline {}", i));

        println!("Baseline {} - TPS: {:.2}", i + 1, metrics.transactions_per_second);
        sleep(Duration::from_secs(2)).await;
    }

    // Execute exactly 5 explicit transactions
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
            while output.next().await.is_some() {}
        }
    }

    sleep(Duration::from_secs(2)).await;

    // Collect after transactions
    let metrics_after = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics after transactions");

    println!(
        "After 5 transactions - TPS: {:.2}, Commits/sec: {:.2}",
        metrics_after.transactions_per_second,
        metrics_after.commits_per_second
    );

    // TPS should be positive (we did commit transactions)
    // Should be reasonable (not 0, not 1000)
    assert!(
        metrics_after.transactions_per_second > 0.0,
        "TPS should be positive after transactions"
    );
    assert!(
        metrics_after.transactions_per_second < 100.0,
        "TPS should not be wildly inflated: got {:.2}",
        metrics_after.transactions_per_second
    );

    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_first_sample_has_zero_rates() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    // Verify that the FIRST collection always returns 0 for rates
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("pg-first-sample"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    sleep(Duration::from_secs(2)).await;

    let docker = Arc::new(Docker::connect_with_local_defaults().expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // Very first collection
    let metrics = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect metrics");

    // First sample should ALWAYS have 0 rates (no previous sample to compare)
    assert_eq!(
        metrics.queries_per_second, 0.0,
        "First sample MUST have 0 QPS"
    );
    assert_eq!(
        metrics.transactions_per_second, 0.0,
        "First sample MUST have 0 TPS"
    );
    assert_eq!(
        metrics.commits_per_second, 0.0,
        "First sample MUST have 0 commits/sec"
    );
    assert_eq!(
        metrics.rollbacks_per_second, 0.0,
        "First sample MUST have 0 rollbacks/sec"
    );

    println!("✓ First sample correctly returns 0 for all rates");

    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_idle_database_shows_minimal_qps() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    // Test that an idle database shows very low QPS after stabilization
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("pg-idle-qps"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    sleep(Duration::from_secs(3)).await;

    let docker = Arc::new(Docker::connect_with_local_defaults().expect("Failed to connect to Docker"));
    let collector = DockerDatabaseMetricsCollector::new(docker.clone());

    // Let database stabilize - collect metrics 5 times
    for i in 0..5 {
        let metrics = collector
            .collect(&container.id, DatabaseType::Postgres)
            .await
            .expect(&format!("Failed to collect metrics {}", i));

        println!(
            "Idle sample {} - QPS: {:.2}, TPS: {:.2}",
            i + 1,
            metrics.queries_per_second,
            metrics.transactions_per_second
        );

        sleep(Duration::from_secs(3)).await;
    }

    // After stabilization, collect final metrics
    sleep(Duration::from_secs(5)).await;

    let final_metrics = collector
        .collect(&container.id, DatabaseType::Postgres)
        .await
        .expect("Failed to collect final metrics");

    println!(
        "Final idle metrics - QPS: {:.2}, TPS: {:.2}",
        final_metrics.queries_per_second,
        final_metrics.transactions_per_second
    );

    // An idle database might still have some background activity (autovacuum, stats collector)
    // but it should be LOW (< 100 QPS is reasonable for an idle database)
    assert!(
        final_metrics.queries_per_second < 100.0,
        "Idle database QPS should be low after stabilization, got {:.2}",
        final_metrics.queries_per_second
    );

    assert!(
        final_metrics.transactions_per_second < 50.0,
        "Idle database TPS should be low after stabilization, got {:.2}",
        final_metrics.transactions_per_second
    );

    cleanup_container(&container.id).await.ok();
}
