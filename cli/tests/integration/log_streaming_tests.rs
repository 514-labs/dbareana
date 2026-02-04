//! Integration tests for log streaming functionality
//! Tests require Docker to be running

use dbarena::container::{ContainerConfig, DatabaseType};
use dbarena::monitoring::LogStreamer;
use bollard::Docker;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use futures::StreamExt;

use crate::common::{create_and_start_container, cleanup_container, unique_container_name, docker_available};

#[tokio::test]
#[ignore] // Requires Docker
async fn test_log_streaming_postgres() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("postgres-log-test"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create test container");

    sleep(Duration::from_secs(2)).await;

    let docker = Docker::connect_with_local_defaults().expect("Failed to connect to Docker");
    let streamer = LogStreamer::new(Arc::new(docker));

    let logs = streamer.fetch_recent_logs(&container.id, 50).await.expect("Failed to fetch logs");

    assert!(!logs.is_empty(), "Should have retrieved some log lines");
    let log_text = logs.join("\n");
    assert!(
        log_text.contains("PostgreSQL") || log_text.contains("database system"),
        "Logs should contain PostgreSQL-related content"
    );

    cleanup_container(&container.id).await.ok();
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_log_ansi_code_stripping() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("postgres-ansi-test"))
        .with_version("16".to_string());

    let container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create test container");

    sleep(Duration::from_secs(2)).await;

    let docker = Docker::connect_with_local_defaults().expect("Failed to connect to Docker");
    let streamer = LogStreamer::new(Arc::new(docker));

    let logs = streamer.fetch_recent_logs(&container.id, 20).await.expect("Failed to fetch logs");

    for log in &logs {
        assert!(
            !log.contains('\x1b'),
            "Log lines should not contain ANSI escape codes"
        );
    }

    cleanup_container(&container.id).await.ok();
}
