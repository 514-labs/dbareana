/// Integration tests for network commands
/// Run with: cargo test --test integration -- --ignored

use std::time::Duration;

use dbarena::container::{ContainerConfig, DatabaseType, DockerClient};
use dbarena::network::{NetworkConfig, NetworkManager};

#[path = "../common/mod.rs"]
mod common;
use common::{create_and_start_container, cleanup_container, docker_available, unique_container_name};

#[tokio::test]
#[ignore]
async fn test_network_lifecycle() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let network_name = format!("test-network-{}", uuid::Uuid::new_v4().to_string());

    let client = DockerClient::new().expect("Failed to create docker client");
    let manager = NetworkManager::new(client.clone());

    let config = NetworkConfig::new(network_name.clone());
    let network = manager
        .create_network(config)
        .await
        .expect("Failed to create network");

    assert_eq!(network.name, network_name);

    let networks = manager
        .list_networks(true)
        .await
        .expect("Failed to list networks");
    assert!(
        networks.iter().any(|n| n.name == network_name),
        "Created network should be listed"
    );

    let inspected = manager
        .inspect_network(&network_name)
        .await
        .expect("Failed to inspect network");
    assert_eq!(inspected.name, network_name);

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-network-container"));
    let test_container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    manager
        .connect_container(&network_name, &test_container.id, None)
        .await
        .expect("Failed to connect container to network");

    manager
        .disconnect_container(&network_name, &test_container.id)
        .await
        .expect("Failed to disconnect container from network");

    manager
        .delete_network(&network_name)
        .await
        .expect("Failed to delete network");

    cleanup_container(&test_container.id)
        .await
        .expect("Failed to cleanup container");
}
