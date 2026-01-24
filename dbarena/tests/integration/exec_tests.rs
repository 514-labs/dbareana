/// Integration tests for exec command
/// These tests require Docker to be running
///
/// Run with: cargo test --test integration -- --ignored

use dbarena::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
use std::fs;
use std::time::Duration;

#[path = "../common/mod.rs"]
mod common;
use common::{
    create_and_start_container, docker_available, execute_query, tempdir, unique_container_name,
};

// ============================================================================
// Execute Inline SQL Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_exec_inline_sql_postgres() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-exec-inline"))
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "postgres".to_string())
        .with_env_var("POSTGRES_PASSWORD".to_string(), "postgres".to_string());

    let test_container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    // Execute inline SQL
    let create_table = "CREATE TABLE exec_test (id SERIAL PRIMARY KEY, name VARCHAR(50));";
    let result = execute_query(&test_container.id, create_table, DatabaseType::Postgres)
        .await
        .expect("Failed to execute SQL");

    // Should succeed (may have CREATE TABLE in output)
    assert!(!result.contains("ERROR"), "SQL should execute without error");

    // Insert data
    let insert = "INSERT INTO exec_test (name) VALUES ('test1'), ('test2');";
    execute_query(&test_container.id, insert, DatabaseType::Postgres)
        .await
        .expect("Failed to insert data");

    // Query data
    let select = "SELECT COUNT(*) FROM exec_test;";
    let result = execute_query(&test_container.id, select, DatabaseType::Postgres)
        .await
        .expect("Failed to query data");

    assert!(result.contains("2") || result.contains("(2 rows)"), "Should have 2 rows");
}

#[tokio::test]
#[ignore]
async fn test_exec_inline_sql_mysql() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::MySQL)
        .with_name(unique_container_name("test-exec-mysql"))
        .with_env_var("MYSQL_ROOT_PASSWORD".to_string(), "mysql".to_string())
        .with_env_var("MYSQL_DATABASE".to_string(), "testdb".to_string());

    let test_container = create_and_start_container(config, Duration::from_secs(90))
        .await
        .expect("Failed to create container");

    // Execute inline SQL
    let create_table = "CREATE TABLE exec_mysql (id INT AUTO_INCREMENT PRIMARY KEY, data VARCHAR(100));";
    let result = execute_query(&test_container.id, create_table, DatabaseType::MySQL)
        .await
        .expect("Failed to execute SQL");

    // Should succeed
    assert!(!result.contains("ERROR"), "SQL should execute without error");
}

#[tokio::test]
#[ignore]
async fn test_exec_inline_sql_error() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-exec-error"))
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "postgres".to_string())
        .with_env_var("POSTGRES_PASSWORD".to_string(), "postgres".to_string());

    let test_container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    // Execute invalid SQL
    let invalid_sql = "INVALID SQL SYNTAX HERE;";
    let result = execute_query(&test_container.id, invalid_sql, DatabaseType::Postgres).await;

    // Should get an error (either from Rust or in the output)
    assert!(
        result.is_err() || result.unwrap().contains("ERROR"),
        "Invalid SQL should produce an error"
    );
}

// ============================================================================
// Execute from File Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_exec_from_file_postgres() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("exec_script.sql");

    let script_content = r#"
CREATE TABLE file_test (
    id SERIAL PRIMARY KEY,
    title VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO file_test (title) VALUES ('First Post');
INSERT INTO file_test (title) VALUES ('Second Post');
INSERT INTO file_test (title) VALUES ('Third Post');
    "#;

    fs::write(&script_path, script_content).expect("Failed to write script");

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-exec-file"))
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "postgres".to_string())
        .with_env_var("POSTGRES_PASSWORD".to_string(), "postgres".to_string());

    let test_container = create_and_start_container(config.clone(), Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    // Execute the script using init script executor (which is what exec command uses internally)
    let client = DockerClient::new().expect("Failed to create Docker client");
    let log_manager = dbarena::init::LogManager::new(Some(temp_dir.path().join("logs")))
        .expect("Failed to create log manager");

    let results = dbarena::init::execute_init_scripts(
        client.docker(),
        &test_container.id,
        vec![script_path],
        DatabaseType::Postgres,
        &config,
        false,
        &log_manager,
    )
    .await
    .expect("Failed to execute script");

    assert_eq!(results.len(), 1);
    assert!(results[0].success, "Script should execute successfully");

    // Verify data
    let query = "SELECT COUNT(*) FROM file_test;";
    let result = execute_query(&test_container.id, query, DatabaseType::Postgres)
        .await
        .expect("Failed to query");

    assert!(result.contains("3") || result.contains("(3 rows)"), "Should have 3 rows");
}

#[tokio::test]
#[ignore]
async fn test_exec_file_not_found() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-exec-notfound"))
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "postgres".to_string())
        .with_env_var("POSTGRES_PASSWORD".to_string(), "postgres".to_string());

    let test_container = create_and_start_container(config.clone(), Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    let nonexistent_path = std::path::PathBuf::from("/nonexistent/script.sql");

    let client = DockerClient::new().expect("Failed to create Docker client");
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let log_manager = dbarena::init::LogManager::new(Some(temp_dir.path().join("logs")))
        .expect("Failed to create log manager");

    let result = dbarena::init::execute_init_scripts(
        client.docker(),
        &test_container.id,
        vec![nonexistent_path],
        DatabaseType::Postgres,
        &config,
        false,
        &log_manager,
    )
    .await;

    // Should fail because file doesn't exist
    assert!(result.is_err(), "Should fail when script file doesn't exist");
}

// ============================================================================
// Container Selection Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_exec_on_stopped_container() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-exec-stopped"));

    let test_container = common::create_test_container(config)
        .await
        .expect("Failed to create container");

    // Don't start the container - it's stopped

    // Try to execute SQL on stopped container
    let query = "SELECT 1;";
    let result = execute_query(&test_container.id, query, DatabaseType::Postgres).await;

    // Should fail because container is not running
    assert!(result.is_err(), "Should fail when container is stopped");
}

#[tokio::test]
#[ignore]
async fn test_exec_list_running_containers() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    // Create and start multiple containers
    let config1 = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-exec-list-1"))
        .with_port(15440);

    let config2 = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-exec-list-2"))
        .with_port(15441);

    let container1 = create_and_start_container(config1, Duration::from_secs(60))
        .await
        .expect("Failed to create container 1");

    let container2 = create_and_start_container(config2, Duration::from_secs(60))
        .await
        .expect("Failed to create container 2");

    // List running containers
    let containers = container1
        .manager
        .list_containers(false) // only running
        .await
        .expect("Failed to list containers");

    // Should find both containers
    assert!(
        containers.iter().any(|c| c.id == container1.id),
        "Should find container 1"
    );
    assert!(
        containers.iter().any(|c| c.id == container2.id),
        "Should find container 2"
    );
}

// ============================================================================
// Multiple Query Execution Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_exec_multiple_queries_same_container() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-exec-multiple"))
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "postgres".to_string())
        .with_env_var("POSTGRES_PASSWORD".to_string(), "postgres".to_string());

    let test_container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    // Execute multiple queries sequentially
    execute_query(
        &test_container.id,
        "CREATE TABLE multi (id SERIAL PRIMARY KEY, value INT);",
        DatabaseType::Postgres,
    )
    .await
    .expect("Failed to create table");

    execute_query(
        &test_container.id,
        "INSERT INTO multi (value) VALUES (10), (20), (30);",
        DatabaseType::Postgres,
    )
    .await
    .expect("Failed to insert data");

    let result = execute_query(
        &test_container.id,
        "SELECT SUM(value) FROM multi;",
        DatabaseType::Postgres,
    )
    .await
    .expect("Failed to sum values");

    // Should contain 60 (10 + 20 + 30)
    assert!(result.contains("60"), "Sum should be 60");
}

// ============================================================================
// Transaction Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_exec_transaction_rollback() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-exec-transaction"))
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "postgres".to_string())
        .with_env_var("POSTGRES_PASSWORD".to_string(), "postgres".to_string());

    let test_container = create_and_start_container(config, Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    // Create table
    execute_query(
        &test_container.id,
        "CREATE TABLE trans_test (id SERIAL PRIMARY KEY, data VARCHAR(50));",
        DatabaseType::Postgres,
    )
    .await
    .expect("Failed to create table");

    // Transaction with rollback
    let transaction = r#"
BEGIN;
INSERT INTO trans_test (data) VALUES ('will be rolled back');
ROLLBACK;
    "#;

    execute_query(&test_container.id, transaction, DatabaseType::Postgres)
        .await
        .expect("Failed to execute transaction");

    // Verify data was not inserted
    let result = execute_query(
        &test_container.id,
        "SELECT COUNT(*) FROM trans_test;",
        DatabaseType::Postgres,
    )
    .await
    .expect("Failed to count");

    assert!(result.contains("0") || result.contains("(0 rows)"), "Should have 0 rows after rollback");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_exec_nonexistent_container() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let result = execute_query(
        "nonexistent-container-id-12345",
        "SELECT 1;",
        DatabaseType::Postgres,
    )
    .await;

    assert!(result.is_err(), "Should fail for nonexistent container");
}
