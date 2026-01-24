/// Integration tests for v0.2.0 config features
/// These tests require Docker to be running
///
/// Run with: cargo test --test integration -- --ignored

use dbarena::config::loader::{find_config_file, load_config};
use dbarena::config::merger::merge_configs;
use dbarena::config::profile::resolve_profile;
use dbarena::container::{ContainerConfig, DatabaseType, DockerClient};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[path = "../common/mod.rs"]
mod common;
use common::{create_test_container, docker_available, tempdir, unique_container_name};

// ============================================================================
// Config File Loading Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_load_toml_config_file() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("dbarena.toml");

    let config_content = r#"
[databases.postgres]
version = "16"

[databases.postgres.env]
POSTGRES_DB = "testdb"
POSTGRES_USER = "testuser"
POSTGRES_PASSWORD = "testpass"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config file");

    let config = load_config(&config_path).expect("Failed to load config");

    assert!(config.databases.contains_key("postgres"));
    let pg_config = &config.databases["postgres"];
    assert_eq!(pg_config.env.get("POSTGRES_DB"), Some(&"testdb".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_load_yaml_config_file() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("dbarena.yaml");

    let config_content = r#"
databases:
  postgres:
    version: "16"
    env:
      POSTGRES_DB: "testdb"
      POSTGRES_USER: "testuser"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config file");

    let config = load_config(&config_path).expect("Failed to load config");

    assert!(config.databases.contains_key("postgres"));
}

#[tokio::test]
#[ignore]
async fn test_config_file_discovery_project_local() {
    // Create a temporary directory and set it as the current directory
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("dbarena.toml");

    fs::write(&config_path, "[defaults]\npersistent = true").expect("Failed to write config");

    // Change to temp directory
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change dir");

    let found = find_config_file().expect("Failed to find config");

    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore dir");

    assert!(found.is_some(), "Should find project-local config");
    assert!(found.unwrap().ends_with("dbarena.toml"));
}

#[tokio::test]
#[ignore]
async fn test_config_with_profile_creates_container() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("dbarena.toml");

    let config_content = r#"
[profiles.dev]
[profiles.dev.env]
POSTGRES_DB = "devdb"
POSTGRES_USER = "devuser"
POSTGRES_PASSWORD = "devpass"

[databases.postgres.profiles.dev]
[databases.postgres.profiles.dev.env]
POSTGRES_INITDB_ARGS = "--encoding=UTF8"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config file");

    let config = load_config(&config_path).expect("Failed to load config");
    let env_vars = resolve_profile(&config, "dev", DatabaseType::Postgres)
        .expect("Failed to resolve profile");

    // Create container with profile env vars
    let mut container_config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-config-profile"))
        .with_env_vars(env_vars);

    let test_container = create_test_container(container_config)
        .await
        .expect("Failed to create container");

    // Inspect container to verify env vars were applied
    let client = DockerClient::new().expect("Failed to create Docker client");
    let inspect = client
        .docker()
        .inspect_container(&test_container.id, None)
        .await
        .expect("Failed to inspect container");

    let container_env = inspect.config.and_then(|c| c.env).unwrap_or_default();

    assert!(
        container_env.iter().any(|e| e.contains("POSTGRES_DB=devdb")),
        "Profile env var should be applied"
    );
}

// ============================================================================
// Profile Resolution Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_profile_precedence_database_overrides_global() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("dbarena.toml");

    let config_content = r#"
[profiles.dev]
[profiles.dev.env]
LOG_LEVEL = "debug"
DB_NAME = "global_db"

[databases.postgres.profiles.dev]
[databases.postgres.profiles.dev.env]
DB_NAME = "postgres_specific_db"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config file");

    let config = load_config(&config_path).expect("Failed to load config");
    let env_vars = resolve_profile(&config, "dev", DatabaseType::Postgres)
        .expect("Failed to resolve profile");

    assert_eq!(env_vars.get("LOG_LEVEL"), Some(&"debug".to_string()));
    assert_eq!(
        env_vars.get("DB_NAME"),
        Some(&"postgres_specific_db".to_string()),
        "Database-specific profile should override global"
    );
}

#[tokio::test]
#[ignore]
async fn test_profile_not_found_suggests_similar() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("dbarena.toml");

    let config_content = r#"
[profiles.development]
[profiles.development.env]
LOG_LEVEL = "debug"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config file");

    let config = load_config(&config_path).expect("Failed to load config");
    let result = resolve_profile(&config, "developmen", DatabaseType::Postgres);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Did you mean 'development'?"),
        "Should suggest similar profile name"
    );
}

// ============================================================================
// Config Merging Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_merge_multiple_config_files() {
    let temp_dir = tempdir().expect("Failed to create temp dir");

    // Base config
    let base_path = temp_dir.path().join("base.toml");
    let base_content = r#"
[defaults]
persistent = false
memory_mb = 256

[databases.postgres.env]
POSTGRES_USER = "baseuser"
POSTGRES_DB = "basedb"
    "#;
    fs::write(&base_path, base_content).expect("Failed to write base config");

    // Override config
    let override_path = temp_dir.path().join("override.toml");
    let override_content = r#"
[defaults]
memory_mb = 512

[databases.postgres.env]
POSTGRES_DB = "overridedb"
POSTGRES_PASSWORD = "secret"
    "#;
    fs::write(&override_path, override_content).expect("Failed to write override config");

    let base_config = load_config(&base_path).expect("Failed to load base config");
    let override_config = load_config(&override_path).expect("Failed to load override config");

    let merged = merge_configs(base_config, override_config);

    // Check defaults merged correctly
    assert_eq!(merged.defaults.persistent, Some(false)); // From base
    assert_eq!(merged.defaults.memory_mb, Some(512)); // Overridden

    // Check database env merged correctly
    let pg_env = &merged.databases["postgres"].env;
    assert_eq!(pg_env.get("POSTGRES_USER"), Some(&"baseuser".to_string())); // From base
    assert_eq!(pg_env.get("POSTGRES_DB"), Some(&"overridedb".to_string())); // Overridden
    assert_eq!(pg_env.get("POSTGRES_PASSWORD"), Some(&"secret".to_string())); // New
}

// ============================================================================
// CLI Override Precedence Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cli_env_overrides_config() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("dbarena.toml");

    let config_content = r#"
[databases.postgres.env]
POSTGRES_DB = "configdb"
POSTGRES_USER = "configuser"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config file");

    let config = load_config(&config_path).expect("Failed to load config");

    // Simulate CLI override
    let mut container_config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(unique_container_name("test-cli-override"));

    // Start with config env vars
    if let Some(pg_config) = config.databases.get("postgres") {
        container_config = container_config.with_env_vars(pg_config.env.clone());
    }

    // Apply CLI override (highest precedence)
    container_config = container_config
        .with_env_var("POSTGRES_DB".to_string(), "clidb".to_string());

    let test_container = create_test_container(container_config)
        .await
        .expect("Failed to create container");

    // Verify CLI override was applied
    let client = DockerClient::new().expect("Failed to create Docker client");
    let inspect = client
        .docker()
        .inspect_container(&test_container.id, None)
        .await
        .expect("Failed to inspect container");

    let container_env = inspect.config.and_then(|c| c.env).unwrap_or_default();

    assert!(
        container_env.iter().any(|e| e.contains("POSTGRES_DB=clidb")),
        "CLI override should take precedence"
    );
}

// ============================================================================
// Env File Loading Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_load_env_file() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let env_path = temp_dir.path().join("test.env");

    let env_content = r#"
POSTGRES_DB=envdb
POSTGRES_USER=envuser
POSTGRES_PASSWORD=envpass
# This is a comment
POSTGRES_INITDB_ARGS=--encoding=UTF8
    "#;

    fs::write(&env_path, env_content).expect("Failed to write env file");

    // Parse env file manually (dbarena doesn't have built-in env file parser yet)
    let env_vars: std::collections::HashMap<String, String> = fs::read_to_string(&env_path)
        .expect("Failed to read env file")
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
        .filter_map(|line| {
            let mut parts = line.splitn(2, '=');
            let key = parts.next()?.trim().to_string();
            let value = parts.next()?.trim().to_string();
            Some((key, value))
        })
        .collect();

    assert_eq!(env_vars.get("POSTGRES_DB"), Some(&"envdb".to_string()));
    assert_eq!(env_vars.get("POSTGRES_USER"), Some(&"envuser".to_string()));
    assert_eq!(env_vars.get("POSTGRES_PASSWORD"), Some(&"envpass".to_string()));
    assert_eq!(
        env_vars.get("POSTGRES_INITDB_ARGS"),
        Some(&"--encoding=UTF8".to_string())
    );
}

// ============================================================================
// Config Validation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_invalid_config_file_format() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("invalid.toml");

    let invalid_content = r#"
[databases.postgres
version = "16"
    "#;

    fs::write(&config_path, invalid_content).expect("Failed to write invalid config");

    let result = load_config(&config_path);
    assert!(result.is_err(), "Loading invalid config should fail");
}

#[tokio::test]
#[ignore]
async fn test_config_with_init_scripts() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("dbarena.toml");

    // Create a dummy SQL script
    let script_path = temp_dir.path().join("init.sql");
    fs::write(&script_path, "CREATE TABLE test (id INT);").expect("Failed to write script");

    let config_content = format!(
        r#"
[databases.postgres]
version = "16"
init_scripts = ["{}"]

[databases.postgres.env]
POSTGRES_DB = "testdb"
    "#,
        script_path.display()
    );

    fs::write(&config_path, config_content).expect("Failed to write config file");

    let config = load_config(&config_path).expect("Failed to load config");

    let pg_config = config.databases.get("postgres").expect("No postgres config");
    assert_eq!(pg_config.init_scripts.len(), 1);
    assert_eq!(pg_config.init_scripts[0].path(), script_path.to_str().unwrap());
}

// ============================================================================
// Config Defaults Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_config_defaults_applied() {
    if !docker_available().await {
        eprintln!("Skipping test: Docker not available");
        return;
    }

    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("dbarena.toml");

    let config_content = r#"
[defaults]
persistent = true
memory_mb = 512
cpu_shares = 1024
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config file");

    let config = load_config(&config_path).expect("Failed to load config");

    assert_eq!(config.defaults.persistent, Some(true));
    assert_eq!(config.defaults.memory_mb, Some(512));
    assert_eq!(config.defaults.cpu_shares, Some(1024));

    // These defaults would be applied when creating a container
    // (actual application logic is in the CLI, not tested here)
}
