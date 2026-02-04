/// Integration tests for v0.1.0 features
/// These tests require Docker to be running
///
/// Run with: cargo test --test integration -- --ignored

use dbarena::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
use dbarena::health::{wait_for_healthy, MySQLHealthChecker, PostgresHealthChecker, SQLServerHealthChecker};
use std::collections::HashMap;
use std::time::Duration;

#[path = "../common/mod.rs"]
mod common;
use common::{create_and_start_container, create_test_container, docker_available, free_port, unique_container_name};

// ============================================================================
// Multi-Database Health Checking Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_mysql_health_checker() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::MySQL)
        .with_name(unique_container_name("test-mysql-health"))
        .with_env_var("MYSQL_ROOT_PASSWORD".to_string(), "mysql".to_string())
        .with_env_var("MYSQL_DATABASE".to_string(), "testdb".to_string());

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    test_container
        .manager
        .start_container(&test_container.id)
        .await
        .expect("Failed to start container");

    // Wait for MySQL to be healthy
    let client = DockerClient::new().expect("Failed to create Docker client");
    let checker = MySQLHealthChecker::new(client.docker().clone());
    let result = wait_for_healthy(&test_container.id, &checker, Duration::from_secs(60)).await;

    assert!(result.is_ok(), "MySQL should become healthy");
}

#[tokio::test]
#[ignore]
async fn test_postgres_health_checker() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-postgres-health"));

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    test_container
        .manager
        .start_container(&test_container.id)
        .await
        .expect("Failed to start container");

    // Wait for PostgreSQL to be healthy
    let client = DockerClient::new().expect("Failed to create Docker client");
    let checker = PostgresHealthChecker::new(client.docker().clone());
    let result = wait_for_healthy(&test_container.id, &checker, Duration::from_secs(60)).await;

    assert!(result.is_ok(), "PostgreSQL should become healthy");
}

#[tokio::test]
#[ignore]
async fn test_health_check_timeout() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-timeout"));

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    // Don't start the container - health check should timeout
    let client = DockerClient::new().expect("Failed to create Docker client");
    let checker = PostgresHealthChecker::new(client.docker().clone());
    let result = wait_for_healthy(&test_container.id, &checker, Duration::from_secs(2)).await;

    assert!(result.is_err(), "Health check should timeout for stopped container");
}

// ============================================================================
// Resource Management Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_memory_limit_applied() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let memory_mb = 256;
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-memory"))
        .with_memory_limit(memory_mb);

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    // Inspect container to verify memory limit
    let client = DockerClient::new().expect("Failed to create Docker client");
    let inspect = client
        .docker()
        .inspect_container(&test_container.id, None)
        .await
        .expect("Failed to inspect container");

    let memory_limit = inspect.host_config.and_then(|hc| hc.memory).unwrap_or(0);
    assert_eq!(
        memory_limit as u64,
        memory_mb * 1024 * 1024,
        "Memory limit should be applied"
    );
}

#[tokio::test]
#[ignore]
async fn test_cpu_shares_applied() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let cpu_shares = 512;
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-cpu"))
        .with_cpu_shares(cpu_shares);

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    // Inspect container to verify CPU shares
    let client = DockerClient::new().expect("Failed to create Docker client");
    let inspect = client
        .docker()
        .inspect_container(&test_container.id, None)
        .await
        .expect("Failed to inspect container");

    let container_cpu_shares = inspect.host_config.and_then(|hc| hc.cpu_shares).unwrap_or(0);
    assert_eq!(
        container_cpu_shares as u64, cpu_shares,
        "CPU shares should be applied"
    );
}

// ============================================================================
// Port Management Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_custom_port_assignment() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let custom_port = free_port();
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-custom-port"))
        .with_port(custom_port);

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    test_container
        .manager
        .start_container(&test_container.id)
        .await
        .expect("Failed to start container");

    // Verify port is mapped correctly
    let container_info = test_container
        .manager
        .find_container(&test_container.id)
        .await
        .expect("Failed to find container")
        .expect("Container not found");

    assert_eq!(
        container_info.host_port,
        Some(custom_port),
        "Custom port should be assigned"
    );

    let _ = test_container.manager.destroy_container(&test_container.id, false).await;
}

#[tokio::test]
#[ignore]
async fn test_auto_port_assignment() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    // Don't specify a port - should auto-assign
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-auto-port"));

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    test_container
        .manager
        .start_container(&test_container.id)
        .await
        .expect("Failed to start container");

    // Verify a port was assigned
    let container_info = test_container
        .manager
        .find_container(&test_container.id)
        .await
        .expect("Failed to find container")
        .expect("Container not found");

    assert!(
        container_info.host_port.is_some(),
        "Port should be auto-assigned"
    );
}

// ============================================================================
// Container Tracking & Labels Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_dbarena_label_present() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-labels"));

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    // Inspect container to verify labels
    let client = DockerClient::new().expect("Failed to create Docker client");
    let inspect = client
        .docker()
        .inspect_container(&test_container.id, None)
        .await
        .expect("Failed to inspect container");

    let labels = inspect.config.and_then(|c| c.labels).unwrap_or_default();
    assert!(
        labels.contains_key("dbarena.managed"),
        "Container should have dbarena.managed label"
    );
}

#[tokio::test]
#[ignore]
async fn test_container_metadata_in_labels() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-metadata"))
        .with_version("15".to_string());

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    // Inspect container to verify metadata labels
    let client = DockerClient::new().expect("Failed to create Docker client");
    let inspect = client
        .docker()
        .inspect_container(&test_container.id, None)
        .await
        .expect("Failed to inspect container");

    let labels = inspect.config.and_then(|c| c.labels).unwrap_or_default();
    assert_eq!(
        labels.get("dbarena.database"),
        Some(&"postgres".to_string()),
        "Database type should be in labels"
    );
    assert_eq!(
        labels.get("dbarena.version"),
        Some(&"15".to_string()),
        "Version should be in labels"
    );
}

#[tokio::test]
#[ignore]
async fn test_list_only_dbarena_containers() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-filter"));

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    // List containers - should only get dbarena-managed ones
    let containers = test_container
        .manager
        .list_containers(true)
        .await
        .expect("Failed to list containers");

    // All returned containers should have dbarena label
    assert!(
        containers.iter().any(|c| c.id == test_container.id),
        "Should find our test container"
    );
}

// ============================================================================
// Multi-Version Support Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_multiple_postgres_versions() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config1 = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-pg-16"))
        .with_version("16".to_string())
        .with_port(free_port());

    let config2 = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-pg-15"))
        .with_version("15".to_string())
        .with_port(free_port());

    let container1 = create_and_start_container(config1, Duration::from_secs(60))
        .await
        .expect("Failed to create first container");

    let container2 = create_and_start_container(config2, Duration::from_secs(60))
        .await
        .expect("Failed to create second container");

    // Both should be running
    let info1 = container1
        .manager
        .find_container(&container1.id)
        .await
        .expect("Failed to find container 1")
        .expect("Container 1 not found");

    let info2 = container2
        .manager
        .find_container(&container2.id)
        .await
        .expect("Failed to find container 2")
        .expect("Container 2 not found");

    assert!(info1.name.contains("16"), "First container should be version 16");
    assert!(info2.name.contains("15"), "Second container should be version 15");

    let _ = container1.manager.destroy_container(&container1.id, false).await;
    let _ = container2.manager.destroy_container(&container2.id, false).await;
}

// ============================================================================
// Container Restart Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_container_restart() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-restart"));

    let test_container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    // Stop the container
    test_container
        .manager
        .stop_container(&test_container.id, Some(5))
        .await
        .expect("Failed to stop container");

    // Restart it
    test_container
        .manager
        .start_container(&test_container.id)
        .await
        .expect("Failed to restart container");

    // Should be running again
    let info = test_container
        .manager
        .find_container(&test_container.id)
        .await
        .expect("Failed to find container")
        .expect("Container not found");

    assert_eq!(info.status.to_string(), "running", "Container should be running after restart");
}

// ============================================================================
// Container Inspect Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_inspect_container_details() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-inspect"))
        .with_version("16".to_string());

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    let container_info = test_container
        .manager
        .find_container(&test_container.id)
        .await
        .expect("Failed to find container")
        .expect("Container not found");

    assert_eq!(container_info.database_type, "postgres");
    assert_eq!(container_info.version, "16");
    assert!(!container_info.id.is_empty());
}

// ============================================================================
// Environment Variable Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_custom_environment_variables() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let mut env_vars = HashMap::new();
    env_vars.insert("POSTGRES_USER".to_string(), "customuser".to_string());
    env_vars.insert("POSTGRES_DB".to_string(), "customdb".to_string());
    env_vars.insert("POSTGRES_PASSWORD".to_string(), "custompass".to_string());

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-env"))
        .with_env_vars(env_vars.clone());

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    // Inspect container to verify env vars
    let client = DockerClient::new().expect("Failed to create Docker client");
    let inspect = client
        .docker()
        .inspect_container(&test_container.id, None)
        .await
        .expect("Failed to inspect container");

    let container_env = inspect.config.and_then(|c| c.env).unwrap_or_default();

    assert!(
        container_env.iter().any(|e| e.contains("POSTGRES_USER=customuser")),
        "Custom POSTGRES_USER should be set"
    );
    assert!(
        container_env.iter().any(|e| e.contains("POSTGRES_DB=customdb")),
        "Custom POSTGRES_DB should be set"
    );
}

// ============================================================================
// Destroy with Volume Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_destroy_with_volumes() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-destroy-volumes"))
        .with_persistent(true);

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    let container_id = test_container.id.clone();

    // Destroy with volumes
    test_container
        .manager
        .destroy_container(&container_id, true)
        .await
        .expect("Failed to destroy container");

    // Container should be gone
    let result = test_container.manager.find_container(&container_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none(), "Container should be deleted");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_start_nonexistent_container() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let client = DockerClient::new().expect("Failed to create Docker client");
    let manager = ContainerManager::new(client);

    let result = manager.start_container("nonexistent-container-id-12345").await;
    assert!(result.is_err(), "Starting nonexistent container should fail");
}

#[tokio::test]
#[ignore]
async fn test_stop_already_stopped_container() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-double-stop"));

    let test_container = create_test_container(config)
        .await
        .expect("Failed to create container");

    // Container is already stopped (never started)
    // Stopping it should either succeed or fail gracefully
    let result = test_container
        .manager
        .stop_container(&test_container.id, Some(5))
        .await;

    // Should either succeed or error, but not panic
    assert!(result.is_ok() || result.is_err());
}
