/// Integration tests for config utility commands
/// These tests require Docker to be running
///
/// Run with: cargo test --test integration -- --ignored

use dbarena::config::{load_config, load_or_default, validate_config};
use std::fs;
use std::path::PathBuf;

#[path = "../common/mod.rs"]
mod common;
use common::{docker_available, tempdir};

// ============================================================================
// Config Validate Command Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_config_validate_valid_config() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("valid.toml");

    let config_content = r#"
[defaults]
persistent = false
memory_mb = 512

[databases.postgres.env]
POSTGRES_DB = "testdb"
POSTGRES_USER = "testuser"
POSTGRES_PASSWORD = "testpass"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    let config = load_config(&config_path).expect("Failed to load config");
    let validation = validate_config(&config).expect("Validation should succeed");

    assert!(validation.warnings.is_empty(), "Should have no warnings");
}

#[tokio::test]
#[ignore]
async fn test_config_validate_with_warnings() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("warnings.toml");

    let config_content = r#"
[defaults]
memory_mb = 2000000  # Very large - should generate warning

[databases.postgres.env]
POSTGRES_DB = "testdb"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    let config = load_config(&config_path).expect("Failed to load config");
    let validation = validate_config(&config).expect("Validation should succeed");

    assert!(!validation.warnings.is_empty(), "Should have warnings for low memory");
}

#[tokio::test]
#[ignore]
async fn test_config_validate_check_scripts_exist() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("with_scripts.toml");

    // Create an actual script file
    let script_path = temp_dir.path().join("init.sql");
    fs::write(&script_path, "CREATE TABLE test (id INT);").expect("Failed to write script");

    let config_content = format!(
        r#"
[databases.postgres]
init_scripts = ["{}"]

[databases.postgres.env]
POSTGRES_DB = "testdb"
    "#,
        script_path.display()
    );

    fs::write(&config_path, config_content).expect("Failed to write config");

    let config = load_config(&config_path).expect("Failed to load config");

    // Check that scripts exist
    for (_, db_config) in &config.databases {
        for script in &db_config.init_scripts {
            let path = PathBuf::from(script.path());
            assert!(path.exists(), "Script file should exist: {}", path.display());
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_config_validate_missing_script() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("missing_script.toml");

    let config_content = r#"
[databases.postgres]
init_scripts = ["/nonexistent/script.sql"]

[databases.postgres.env]
POSTGRES_DB = "testdb"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    let config = load_config(&config_path).expect("Failed to load config");

    // Check for missing scripts
    let mut has_missing = false;
    for (_, db_config) in &config.databases {
        for script in &db_config.init_scripts {
            let path = PathBuf::from(script.path());
            if !path.exists() {
                has_missing = true;
            }
        }
    }

    assert!(has_missing, "Should detect missing script file");
}

#[tokio::test]
#[ignore]
async fn test_config_validate_invalid_syntax() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("invalid.toml");

    let invalid_content = r#"
[databases.postgres
version = "16"
    "#;

    fs::write(&config_path, invalid_content).expect("Failed to write config");

    let result = load_config(&config_path);
    assert!(result.is_err(), "Should fail to load invalid config");
}

// ============================================================================
// Config Show Command Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_config_show_displays_defaults() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("show_defaults.toml");

    let config_content = r#"
[defaults]
persistent = true
memory_mb = 1024
cpu_shares = 2048
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    let config = load_config(&config_path).expect("Failed to load config");

    assert_eq!(config.defaults.persistent, Some(true));
    assert_eq!(config.defaults.memory_mb, Some(1024));
    assert_eq!(config.defaults.cpu_shares, Some(2048));
}

#[tokio::test]
#[ignore]
async fn test_config_show_displays_profiles() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("show_profiles.toml");

    let config_content = r#"
[profiles.dev]
[profiles.dev.env]
LOG_LEVEL = "debug"
ENV = "development"

[profiles.prod]
[profiles.prod.env]
LOG_LEVEL = "error"
ENV = "production"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    let config = load_config(&config_path).expect("Failed to load config");

    assert_eq!(config.profiles.len(), 2);
    assert!(config.profiles.contains_key("dev"));
    assert!(config.profiles.contains_key("prod"));

    let dev_env = &config.profiles["dev"].env;
    assert_eq!(dev_env.get("LOG_LEVEL"), Some(&"debug".to_string()));
    assert_eq!(dev_env.get("ENV"), Some(&"development".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_config_show_displays_databases() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("show_databases.toml");

    let config_content = r#"
[databases.postgres]
default_version = "16"

[databases.postgres.env]
POSTGRES_DB = "myapp"
POSTGRES_USER = "appuser"

[databases.mysql]
default_version = "8.0"

[databases.mysql.env]
MYSQL_DATABASE = "myapp"
MYSQL_USER = "appuser"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    let config = load_config(&config_path).expect("Failed to load config");

    assert_eq!(config.databases.len(), 2);
    assert!(config.databases.contains_key("postgres"));
    assert!(config.databases.contains_key("mysql"));

    let pg = &config.databases["postgres"];
    assert_eq!(pg.default_version, Some("16".to_string()));
    assert_eq!(pg.env.get("POSTGRES_DB"), Some(&"myapp".to_string()));

    let mysql = &config.databases["mysql"];
    assert_eq!(mysql.default_version, Some("8.0".to_string()));
    assert_eq!(mysql.env.get("MYSQL_DATABASE"), Some(&"myapp".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_config_show_specific_profile() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("show_profile.toml");

    let config_content = r#"
[profiles.dev]
[profiles.dev.env]
LOG_LEVEL = "debug"

[profiles.staging]
[profiles.staging.env]
LOG_LEVEL = "info"

[profiles.prod]
[profiles.prod.env]
LOG_LEVEL = "error"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    let config = load_config(&config_path).expect("Failed to load config");

    // Show only dev profile
    let dev_profile = config.profiles.get("dev").expect("Dev profile should exist");
    assert_eq!(dev_profile.env.get("LOG_LEVEL"), Some(&"debug".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_config_show_empty_config() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("empty.toml");

    fs::write(&config_path, "").expect("Failed to write config");

    let config = load_config(&config_path).expect("Failed to load config");

    assert!(config.profiles.is_empty());
    assert!(config.databases.is_empty());
    assert_eq!(config.defaults.persistent, None);
}

// ============================================================================
// Config Init Command Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_config_init_generates_example() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("generated.toml");

    // Simulate what config init does - generate example config
    let example_config = r#"# dbarena Configuration File
# This is an example configuration with common settings

[defaults]
# Global defaults applied to all containers
persistent = false
memory_mb = 512
cpu_shares = 1024

[profiles.dev]
# Development profile
[profiles.dev.env]
LOG_LEVEL = "debug"
ENV = "development"

[profiles.prod]
# Production profile
[profiles.prod.env]
LOG_LEVEL = "error"
ENV = "production"

[databases.postgres]
# PostgreSQL configuration
default_version = "16"

[databases.postgres.env]
POSTGRES_DB = "myapp"
POSTGRES_USER = "postgres"
POSTGRES_PASSWORD = "postgres"

# Example with initialization scripts
# init_scripts = ["./scripts/schema.sql", "./scripts/seed.sql"]

[databases.mysql]
# MySQL configuration
default_version = "8.0"

[databases.mysql.env]
MYSQL_DATABASE = "myapp"
MYSQL_ROOT_PASSWORD = "mysql"
MYSQL_USER = "appuser"
MYSQL_PASSWORD = "apppass"
"#;

    fs::write(&config_path, example_config).expect("Failed to write config");

    // Verify it's valid
    let config = load_config(&config_path).expect("Generated config should be valid");

    assert!(config.profiles.contains_key("dev"));
    assert!(config.profiles.contains_key("prod"));
    assert!(config.databases.contains_key("postgres"));
    assert!(config.databases.contains_key("mysql"));
}

// ============================================================================
// Load or Default Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_load_or_default_with_file() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("test.toml");

    let config_content = r#"
[defaults]
persistent = true
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    let config = load_or_default(Some(config_path)).expect("Failed to load config");

    assert_eq!(config.defaults.persistent, Some(true));
}

#[tokio::test]
#[ignore]
async fn test_load_or_default_without_file() {
    let config = load_or_default(None).expect("Failed to load default config");

    // Default config should be empty
    assert!(config.profiles.is_empty());
    assert!(config.databases.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_load_or_default_missing_file() {
    let nonexistent = PathBuf::from("/nonexistent/config.toml");
    let result = load_or_default(Some(nonexistent));

    assert!(result.is_err(), "Should fail when specified file doesn't exist");
}

// ============================================================================
// Config with Init Scripts Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_config_with_multiple_init_scripts() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("multi_scripts.toml");

    // Create script files
    let script1 = temp_dir.path().join("01_schema.sql");
    let script2 = temp_dir.path().join("02_seed.sql");
    let script3 = temp_dir.path().join("03_indexes.sql");

    fs::write(&script1, "CREATE TABLE test1 (id INT);").expect("Failed to write script 1");
    fs::write(&script2, "INSERT INTO test1 VALUES (1);").expect("Failed to write script 2");
    fs::write(&script3, "CREATE INDEX idx ON test1(id);").expect("Failed to write script 3");

    let config_content = format!(
        r#"
[databases.postgres]
init_scripts = ["{}", "{}", "{}"]

[databases.postgres.env]
POSTGRES_DB = "testdb"
    "#,
        script1.display(),
        script2.display(),
        script3.display()
    );

    fs::write(&config_path, config_content).expect("Failed to write config");

    let config = load_config(&config_path).expect("Failed to load config");

    let pg_config = config.databases.get("postgres").expect("Postgres config should exist");
    assert_eq!(pg_config.init_scripts.len(), 3);
}

#[tokio::test]
#[ignore]
async fn test_config_init_script_with_continue_on_error() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("continue_error.toml");

    let script_path = temp_dir.path().join("script.sql");
    fs::write(&script_path, "SELECT 1;").expect("Failed to write script");

    let config_content = format!(
        r#"
[[databases.postgres.init_scripts]]
path = "{}"
continue_on_error = true

[databases.postgres.env]
POSTGRES_DB = "testdb"
    "#,
        script_path.display()
    );

    fs::write(&config_path, config_content).expect("Failed to write config");

    let config = load_config(&config_path).expect("Failed to load config");

    let pg_config = config.databases.get("postgres").expect("Postgres config should exist");
    assert_eq!(pg_config.init_scripts.len(), 1);
    assert!(pg_config.init_scripts[0].continue_on_error());
}

// ============================================================================
// Config Profile Edge Cases
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_config_database_specific_profile_overrides() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("profile_override.toml");

    let config_content = r#"
[profiles.dev]
[profiles.dev.env]
LOG_LEVEL = "debug"
DB_NAME = "global_db"

[databases.postgres.profiles.dev]
[databases.postgres.profiles.dev.env]
DB_NAME = "postgres_specific_db"
POSTGRES_USER = "devuser"
    "#;

    fs::write(&config_path, config_content).expect("Failed to write config");

    let config = load_config(&config_path).expect("Failed to load config");

    // Global profile exists
    assert!(config.profiles.contains_key("dev"));

    // Database-specific profile exists
    let pg_config = config.databases.get("postgres").expect("Postgres config should exist");
    assert!(pg_config.profiles.contains_key("dev"));

    let pg_dev_profile = &pg_config.profiles["dev"];
    assert_eq!(
        pg_dev_profile.env.get("DB_NAME"),
        Some(&"postgres_specific_db".to_string())
    );
}
