/// Integration tests for volume commands
/// Run with: cargo test --test integration -- --ignored

use std::sync::Arc;

use bollard::Docker;
use dbarena::container::{VolumeConfig, VolumeManager};

#[path = "../common/mod.rs"]
mod common;
use common::docker_available;

#[tokio::test]
#[ignore]
async fn test_volume_lifecycle() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let docker = Docker::connect_with_local_defaults().expect("Docker not available");
    let manager = VolumeManager::new(Arc::new(docker));

    let volume_name = format!("test-volume-{}", uuid::Uuid::new_v4().to_string());
    let config = VolumeConfig::new(volume_name.clone(), "/data".to_string());

    let created = manager
        .create(config)
        .await
        .expect("Failed to create volume");
    assert_eq!(created, volume_name);

    let volumes = manager.list(true).await.expect("Failed to list volumes");
    assert!(
        volumes.iter().any(|v| v.name == volume_name),
        "Created volume should be listed"
    );

    let details = manager
        .inspect(&volume_name)
        .await
        .expect("Failed to inspect volume");
    assert_eq!(details.name, volume_name);

    manager
        .delete(&volume_name, true)
        .await
        .expect("Failed to delete volume");
}
