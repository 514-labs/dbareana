use dbarena::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
use crate::common::{docker_available, unique_container_name};

#[tokio::test]
#[ignore] // Requires Docker
async fn test_postgres_container_lifecycle() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    let client = DockerClient::new().expect("Failed to create Docker client");
    client
        .verify_connection()
        .await
        .expect("Docker not available");

    let manager = ContainerManager::new(client);

    // Create a container
    let name = unique_container_name("test-postgres-lifecycle");
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(name.clone());

    let container = manager
        .create_container(config)
        .await
        .expect("Failed to create container");

    assert_eq!(container.name, name);
    assert_eq!(container.database_type, "postgres");

    // Start the container
    manager
        .start_container(&container.id)
        .await
        .expect("Failed to start container");

    // List containers
    let containers = manager
        .list_containers(false)
        .await
        .expect("Failed to list containers");
    assert!(containers.iter().any(|c| c.id == container.id));

    // Stop the container
    manager
        .stop_container(&container.id, Some(5))
        .await
        .expect("Failed to stop container");

    // Destroy the container
    manager
        .destroy_container(&container.id, false)
        .await
        .expect("Failed to destroy container");

    // Verify it's gone
    let containers = manager
        .list_containers(true)
        .await
        .expect("Failed to list containers");
    assert!(!containers.iter().any(|c| c.id == container.id));
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_mysql_container_creation() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    let client = DockerClient::new().expect("Failed to create Docker client");
    client
        .verify_connection()
        .await
        .expect("Docker not available");

    let manager = ContainerManager::new(client);

    let name = unique_container_name("test-mysql-lifecycle");
    let config = ContainerConfig::new(DatabaseType::MySQL)
        .with_name(name.clone());

    let container = manager
        .create_container(config)
        .await
        .expect("Failed to create container");

    assert_eq!(container.name, name);
    assert_eq!(container.database_type, "mysql");

    // Cleanup
    manager
        .destroy_container(&container.id, false)
        .await
        .expect("Failed to destroy container");
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_find_container_by_name() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    let client = DockerClient::new().expect("Failed to create Docker client");
    client
        .verify_connection()
        .await
        .expect("Docker not available");

    let manager = ContainerManager::new(client);

    let name = unique_container_name("test-find-container");
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(name.clone());

    let container = manager
        .create_container(config)
        .await
        .expect("Failed to create container");

    // Find by name
    let found = manager
        .find_container(&name)
        .await
        .expect("Failed to find container")
        .expect("Container not found");

    assert_eq!(found.id, container.id);
    assert_eq!(found.name, name);

    // Find by ID prefix
    let found_by_id = manager
        .find_container(&container.id[..8])
        .await
        .expect("Failed to find container")
        .expect("Container not found");

    assert_eq!(found_by_id.id, container.id);

    // Cleanup
    manager
        .destroy_container(&container.id, false)
        .await
        .expect("Failed to destroy container");
}

#[tokio::test]
async fn test_docker_client_connection() {
    let client = match DockerClient::new() {
        Ok(client) => client,
        Err(_) => {
            eprintln!("Skipping test: Docker not available");
            return;
        }
    };
    let result = client.verify_connection().await;
    if result.is_err() {
        eprintln!("Skipping test: Docker not available");
        return;
    }
    assert!(result.is_ok(), "Docker connection failed");
}

#[tokio::test]
async fn test_container_config_builder() {
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_version("15".to_string())
        .with_name("test-db".to_string())
        .with_port(5433)
        .with_persistent(true)
        .with_memory_limit(512)
        .with_cpu_shares(512);

    assert_eq!(config.database, DatabaseType::Postgres);
    assert_eq!(config.version, "15");
    assert_eq!(config.name, Some("test-db".to_string()));
    assert_eq!(config.port, Some(5433));
    assert_eq!(config.persistent, true);
    assert_eq!(config.memory_limit, Some(512 * 1024 * 1024));
    assert_eq!(config.cpu_shares, Some(512));
}
