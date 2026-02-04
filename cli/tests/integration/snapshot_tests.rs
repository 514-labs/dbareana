/// Integration tests for snapshot lifecycle
/// Run with: cargo test --test integration -- --ignored

use std::sync::Arc;
use std::time::Duration;

use bollard::Docker;
use dbarena::container::{ContainerConfig, DatabaseType, DockerClient, ContainerManager};
use dbarena::snapshot::SnapshotManager;

#[path = "../common/mod.rs"]
mod common;
use common::{create_and_start_container, cleanup_container, docker_available, unique_container_name};

#[tokio::test]
#[ignore]
async fn test_snapshot_lifecycle() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-snapshot"));

    let test_container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    let docker = Docker::connect_with_local_defaults().expect("Docker not available");
    let manager = SnapshotManager::new(Arc::new(docker));

    let snapshot = manager
        .create(
            &test_container.id,
            "test-snapshot".to_string(),
            Some("integration test".to_string()),
            true,
        )
        .await
        .expect("Failed to create snapshot");

    let snapshots = manager.list().await.expect("Failed to list snapshots");
    assert!(
        snapshots.iter().any(|s| s.id == snapshot.id),
        "Snapshot should be listed"
    );

    let inspected = manager
        .inspect(&snapshot.id)
        .await
        .expect("Failed to inspect snapshot");
    assert_eq!(inspected.name, "test-snapshot");

    let restored = manager
        .restore(&snapshot.id, Some(unique_container_name("restored")), None)
        .await
        .expect("Failed to restore snapshot");

    // Cleanup restored container
    let client = DockerClient::new().expect("Failed to create docker client");
    let container_manager = ContainerManager::new(client);
    container_manager
        .destroy_container(&restored.id, false)
        .await
        .expect("Failed to destroy restored container");

    manager
        .delete(&snapshot.id)
        .await
        .expect("Failed to delete snapshot");

    cleanup_container(&test_container.id)
        .await
        .expect("Failed to cleanup container");
}
