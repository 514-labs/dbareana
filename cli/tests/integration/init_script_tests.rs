/// Integration tests for v0.2.0 init script execution
/// These tests require Docker to be running
///
/// Run with: cargo test --test integration -- --ignored

use dbarena::container::{ContainerConfig, DatabaseType, DockerClient};
use dbarena::init::{execute_init_scripts, LogManager};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

#[path = "../common/mod.rs"]
mod common;
use common::{
    create_and_start_container, docker_available, execute_query, tempdir, unique_container_name,
};

// ============================================================================
// PostgreSQL Init Script Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_postgres_init_script_success() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("init.sql");

    let script_content = r#"
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL
);

INSERT INTO users (username, email) VALUES ('alice', 'alice@example.com');
INSERT INTO users (username, email) VALUES ('bob', 'bob@example.com');
    "#;

    fs::write(&script_path, script_content).expect("Failed to write script");

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-postgres-init"))
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "postgres".to_string())
        .with_env_var("POSTGRES_PASSWORD".to_string(), "postgres".to_string())
        .with_init_script(script_path.clone());

    let test_container = create_and_start_container(config.clone(), Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    // Execute init scripts
    let client = DockerClient::new().expect("Failed to create Docker client");
    let log_manager = LogManager::new(Some(temp_dir.path().join("logs")))
        .expect("Failed to create log manager");

    let results = execute_init_scripts(
        client.docker(),
        &test_container.id,
        vec![script_path],
        DatabaseType::Postgres,
        &config,
        false,
        &log_manager,
    )
    .await
    .expect("Failed to execute init scripts");

    assert_eq!(results.len(), 1);
    if !results[0].success {
        eprintln!("Script failed!");
        eprintln!("Output: {}", results[0].output);
        if let Some(ref err) = results[0].error {
            eprintln!("Error: {:?}", err);
        }
    }
    assert!(results[0].success, "Script should execute successfully");

    // Verify data was inserted
    let output = execute_query(&test_container.id, "SELECT COUNT(*) FROM users;", DatabaseType::Postgres)
        .await
        .expect("Failed to query database");

    assert!(output.contains("2") || output.contains("(2 rows)"), "Should have 2 users");
}

#[tokio::test]
#[ignore]
async fn test_postgres_init_script_error() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("error.sql");

    // Script with intentional syntax error
    let script_content = r#"
CREATE TABLE users (
    id SERIAL PRIMARY KEY
    username VARCHAR(50) NOT NULL
);
    "#;

    fs::write(&script_path, script_content).expect("Failed to write script");

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-postgres-error"))
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "postgres".to_string())
        .with_env_var("POSTGRES_PASSWORD".to_string(), "postgres".to_string())
        .with_init_script(script_path.clone());

    let test_container = create_and_start_container(config.clone(), Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    // Execute init scripts
    let client = DockerClient::new().expect("Failed to create Docker client");
    let log_manager = LogManager::new(Some(temp_dir.path().join("logs")))
        .expect("Failed to create log manager");

    let results = execute_init_scripts(
        client.docker(),
        &test_container.id,
        vec![script_path],
        DatabaseType::Postgres,
        &config,
        false,
        &log_manager,
    )
    .await
    .expect("Init script execution should return results even on error");

    assert_eq!(results.len(), 1);
    assert!(!results[0].success, "Script should fail");
    assert!(results[0].error.is_some(), "Should have error details");

    let error = results[0].error.as_ref().unwrap();
    assert!(error.line_number.is_some(), "Should extract line number");
}

// ============================================================================
// MySQL Init Script Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_mysql_init_script_success() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("init.sql");

    let script_content = r#"
CREATE TABLE products (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    price DECIMAL(10, 2)
);

INSERT INTO products (name, price) VALUES ('Widget', 9.99);
INSERT INTO products (name, price) VALUES ('Gadget', 19.99);
    "#;

    fs::write(&script_path, script_content).expect("Failed to write script");

    let config = ContainerConfig::new(DatabaseType::MySQL)
        .with_name(unique_container_name("test-mysql-init"))
        .with_env_var("MYSQL_ROOT_PASSWORD".to_string(), "mysql".to_string())
        .with_env_var("MYSQL_DATABASE".to_string(), "testdb".to_string())
        .with_init_script(script_path.clone());

    let test_container = create_and_start_container(config.clone(), Duration::from_secs(90))
        .await
        .expect("Failed to create container");

    // Execute init scripts
    let client = DockerClient::new().expect("Failed to create Docker client");
    let log_manager = LogManager::new(Some(temp_dir.path().join("logs")))
        .expect("Failed to create log manager");

    let results = execute_init_scripts(
        client.docker(),
        &test_container.id,
        vec![script_path],
        DatabaseType::MySQL,
        &config,
        false,
        &log_manager,
    )
    .await
    .expect("Failed to execute init scripts");

    assert_eq!(results.len(), 1);
    assert!(results[0].success, "Script should execute successfully");
}

#[tokio::test]
#[ignore]
async fn test_mysql_init_script_error() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("error.sql");

    // Script with syntax error (missing column type)
    let script_content = r#"
CREATE TABLE products (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name,
    price DECIMAL(10, 2)
);
    "#;

    fs::write(&script_path, script_content).expect("Failed to write script");

    let config = ContainerConfig::new(DatabaseType::MySQL)
        .with_name(unique_container_name("test-mysql-error"))
        .with_env_var("MYSQL_ROOT_PASSWORD".to_string(), "mysql".to_string())
        .with_env_var("MYSQL_DATABASE".to_string(), "testdb".to_string())
        .with_init_script(script_path.clone());

    let test_container = create_and_start_container(config.clone(), Duration::from_secs(90))
        .await
        .expect("Failed to create container");

    // Execute init scripts
    let client = DockerClient::new().expect("Failed to create Docker client");
    let log_manager = LogManager::new(Some(temp_dir.path().join("logs")))
        .expect("Failed to create log manager");

    let results = execute_init_scripts(
        client.docker(),
        &test_container.id,
        vec![script_path],
        DatabaseType::MySQL,
        &config,
        false,
        &log_manager,
    )
    .await
    .expect("Init script execution should return results");

    assert_eq!(results.len(), 1);
    assert!(!results[0].success, "Script should fail");
    assert!(results[0].error.is_some(), "Should have error details");
}

// ============================================================================
// Multi-Script Execution Order Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_multi_script_execution_order() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");

    // Script 1: Create schema
    let script1_path = temp_dir.path().join("01_schema.sql");
    fs::write(
        &script1_path,
        "CREATE TABLE orders (id SERIAL PRIMARY KEY, total DECIMAL(10, 2));",
    )
    .expect("Failed to write script 1");

    // Script 2: Insert data
    let script2_path = temp_dir.path().join("02_data.sql");
    fs::write(
        &script2_path,
        "INSERT INTO orders (total) VALUES (100.00), (200.00);",
    )
    .expect("Failed to write script 2");

    // Script 3: Add index
    let script3_path = temp_dir.path().join("03_index.sql");
    fs::write(&script3_path, "CREATE INDEX idx_total ON orders(total);")
        .expect("Failed to write script 3");

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-multi-script"))
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "postgres".to_string())
        .with_env_var("POSTGRES_PASSWORD".to_string(), "postgres".to_string())
        .with_init_scripts(vec![
            script1_path.clone(),
            script2_path.clone(),
            script3_path.clone(),
        ]);

    let test_container = create_and_start_container(config.clone(), Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    // Execute init scripts in order
    let client = DockerClient::new().expect("Failed to create Docker client");
    let log_manager = LogManager::new(Some(temp_dir.path().join("logs")))
        .expect("Failed to create log manager");

    let results = execute_init_scripts(
        client.docker(),
        &test_container.id,
        vec![script1_path, script2_path, script3_path],
        DatabaseType::Postgres,
        &config,
        false,
        &log_manager,
    )
    .await
    .expect("Failed to execute init scripts");

    assert_eq!(results.len(), 3);
    assert!(results[0].success, "Script 1 should succeed");
    assert!(results[1].success, "Script 2 should succeed");
    assert!(results[2].success, "Script 3 should succeed");

    // Verify data
    let output = execute_query(&test_container.id, "SELECT COUNT(*) FROM orders;", DatabaseType::Postgres)
        .await
        .expect("Failed to query");

    assert!(output.contains("2") || output.contains("(2 rows)"));
}

// ============================================================================
// Continue-on-Error Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_continue_on_error_false_stops_execution() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");

    // Script 1: Success
    let script1_path = temp_dir.path().join("01_success.sql");
    fs::write(&script1_path, "CREATE TABLE test1 (id SERIAL PRIMARY KEY);")
        .expect("Failed to write script 1");

    // Script 2: Error
    let script2_path = temp_dir.path().join("02_error.sql");
    fs::write(&script2_path, "INVALID SQL SYNTAX HERE;").expect("Failed to write script 2");

    // Script 3: Would succeed but shouldn't run
    let script3_path = temp_dir.path().join("03_skip.sql");
    fs::write(&script3_path, "CREATE TABLE test3 (id SERIAL PRIMARY KEY);")
        .expect("Failed to write script 3");

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-stop-on-error"))
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "postgres".to_string())
        .with_env_var("POSTGRES_PASSWORD".to_string(), "postgres".to_string())
        .with_continue_on_error(false); // Stop on first error

    let test_container = create_and_start_container(config.clone(), Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    let client = DockerClient::new().expect("Failed to create Docker client");
    let log_manager = LogManager::new(Some(temp_dir.path().join("logs")))
        .expect("Failed to create log manager");

    let results = execute_init_scripts(
        client.docker(),
        &test_container.id,
        vec![script1_path, script2_path, script3_path],
        DatabaseType::Postgres,
        &config,
        false, // continue_on_error = false
        &log_manager,
    )
    .await
    .expect("Should return results even with error");

    // Should stop after script 2 fails
    assert_eq!(results.len(), 2, "Should stop after first error");
    assert!(results[0].success, "Script 1 should succeed");
    assert!(!results[1].success, "Script 2 should fail");
}

#[tokio::test]
#[ignore]
async fn test_continue_on_error_true_executes_all() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");

    // Script 1: Success
    let script1_path = temp_dir.path().join("01_success.sql");
    fs::write(&script1_path, "CREATE TABLE test1 (id SERIAL PRIMARY KEY);")
        .expect("Failed to write script 1");

    // Script 2: Error
    let script2_path = temp_dir.path().join("02_error.sql");
    fs::write(&script2_path, "INVALID SQL SYNTAX HERE;").expect("Failed to write script 2");

    // Script 3: Success (should still run)
    let script3_path = temp_dir.path().join("03_continue.sql");
    fs::write(&script3_path, "CREATE TABLE test3 (id SERIAL PRIMARY KEY);")
        .expect("Failed to write script 3");

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-continue-on-error"))
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "postgres".to_string())
        .with_env_var("POSTGRES_PASSWORD".to_string(), "postgres".to_string())
        .with_continue_on_error(true); // Continue even with errors

    let test_container = create_and_start_container(config.clone(), Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    let client = DockerClient::new().expect("Failed to create Docker client");
    let log_manager = LogManager::new(Some(temp_dir.path().join("logs")))
        .expect("Failed to create log manager");

    let results = execute_init_scripts(
        client.docker(),
        &test_container.id,
        vec![script1_path, script2_path, script3_path],
        DatabaseType::Postgres,
        &config,
        true, // continue_on_error = true
        &log_manager,
    )
    .await
    .expect("Should execute all scripts");

    // Should execute all 3 scripts
    assert_eq!(results.len(), 3, "Should execute all scripts");
    assert!(results[0].success, "Script 1 should succeed");
    assert!(!results[1].success, "Script 2 should fail");

    // Debug output for script 3
    if !results[2].success {
        eprintln!("Script 3 failed!");
        eprintln!("Output: {}", results[2].output);
        if let Some(err) = &results[2].error {
            eprintln!("Error: {}", err);
        }
    }
    assert!(results[2].success, "Script 3 should succeed despite script 2 failure");
}

// ============================================================================
// Log Management Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_init_script_logs_created() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("logged_script.sql");

    fs::write(&script_path, "CREATE TABLE logged (id SERIAL);")
        .expect("Failed to write script");

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-init-logs"))
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "postgres".to_string())
        .with_env_var("POSTGRES_PASSWORD".to_string(), "postgres".to_string())
        .with_init_script(script_path.clone());

    let test_container = create_and_start_container(config.clone(), Duration::from_secs(60))
        .await
        .expect("Failed to create container");

    let client = DockerClient::new().expect("Failed to create Docker client");
    let log_dir = temp_dir.path().join("logs");
    let log_manager =
        LogManager::new(Some(log_dir.clone())).expect("Failed to create log manager");

    execute_init_scripts(
        client.docker(),
        &test_container.id,
        vec![script_path.clone()],
        DatabaseType::Postgres,
        &config,
        false,
        &log_manager,
    )
    .await
    .expect("Failed to execute init scripts");

    // Verify log files were created
    assert!(log_dir.exists(), "Log directory should exist");

    // Find the session directory (should be timestamp-based)
    let session_dirs: Vec<_> = fs::read_dir(&log_dir)
        .expect("Failed to read log dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    assert!(!session_dirs.is_empty(), "Should have at least one session directory");

    // Check for metadata file
    let metadata_exists = session_dirs.iter().any(|dir| {
        dir.path().join("metadata.json").exists()
    });
    assert!(metadata_exists, "Metadata file should exist");
}
