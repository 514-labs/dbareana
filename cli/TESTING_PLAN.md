# dbarena v0.2.0 - Comprehensive Testing Plan

## Overview

This document outlines the complete testing strategy for dbarena including all v0.1.0 and v0.2.0 features, with manual test procedures, automated unit tests, integration tests, and error handling procedures.

**Status**: Implementation Complete ✅ | Testing Phase 🔄

**Goal**: Achieve 85%+ test coverage with comprehensive manual and automated testing

## Test Summary

- **Unit Tests**: 40+ tests covering v0.1.0 and v0.2.0
- **Integration Tests**: 35+ tests with Docker
- **Manual Test Cases**: 58 detailed test procedures
  - v0.1.0: 30 tests
  - v0.2.0: 28 tests
- **Performance Tests**: 5 benchmarks
- **Error Recovery Tests**: 3 scenarios

## Testing Strategy

### Test Pyramid
```
         /\
        /  \  Manual & Exploratory Tests (10%)
       /____\
      /      \  Integration Tests (30%)
     /________\
    /          \  Unit Tests (60%)
   /____________\
```

### Testing Phases

1. **Phase 1: Unit Tests** - Test individual components in isolation
2. **Phase 2: Integration Tests** - Test component interactions with Docker
3. **Phase 3: Manual Testing** - User acceptance and edge case testing
4. **Phase 4: Performance Testing** - Verify performance targets
5. **Phase 5: Error Recovery Testing** - Test error handling and recovery

---

## Phase 1: Unit Tests

### 1.0 v0.1.0 Core Functionality Tests

**File**: `tests/unit/v0_1_0_tests.rs`

#### Test: Database Type Parsing

```rust
#[test]
fn test_database_type_from_string() {
    assert_eq!(DatabaseType::from_string("postgres"), Some(DatabaseType::Postgres));
    assert_eq!(DatabaseType::from_string("Postgres"), Some(DatabaseType::Postgres));
    assert_eq!(DatabaseType::from_string("postgresql"), Some(DatabaseType::Postgres));
    assert_eq!(DatabaseType::from_string("mysql"), Some(DatabaseType::MySQL));
    assert_eq!(DatabaseType::from_string("sqlserver"), Some(DatabaseType::SQLServer));
    assert_eq!(DatabaseType::from_string("invalid"), None);
}
```

#### Test: Default Environment Variables

```rust
#[test]
fn test_default_postgres_env_vars() {
    let config = ContainerConfig::new(DatabaseType::Postgres);
    let env_vars = build_env_vars(&config);

    assert!(env_vars.contains(&"POSTGRES_PASSWORD=postgres".to_string()));
    assert!(env_vars.contains(&"POSTGRES_USER=postgres".to_string()));
    assert!(env_vars.contains(&"POSTGRES_DB=testdb".to_string()));
}

#[test]
fn test_default_mysql_env_vars() {
    let config = ContainerConfig::new(DatabaseType::MySQL);
    let env_vars = build_env_vars(&config);

    assert!(env_vars.contains(&"MYSQL_ROOT_PASSWORD=mysql".to_string()));
    assert!(env_vars.contains(&"MYSQL_DATABASE=testdb".to_string()));
}

#[test]
fn test_default_sqlserver_env_vars() {
    let config = ContainerConfig::new(DatabaseType::SQLServer);
    let env_vars = build_env_vars(&config);

    assert!(env_vars.contains(&"ACCEPT_EULA=Y".to_string()));
    assert!(env_vars.contains(&"SA_PASSWORD=YourStrong@Passw0rd".to_string()));
}
```

#### Test: Port Assignment

```rust
#[test]
fn test_default_ports() {
    assert_eq!(ContainerConfig::new(DatabaseType::Postgres).default_port(), 5432);
    assert_eq!(ContainerConfig::new(DatabaseType::MySQL).default_port(), 3306);
    assert_eq!(ContainerConfig::new(DatabaseType::SQLServer).default_port(), 1433);
}

#[test]
fn test_custom_port() {
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_port(5433);

    assert_eq!(config.port, Some(5433));
}
```

#### Test: Resource Limits

```rust
#[test]
fn test_memory_limit_conversion() {
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_memory_limit(512); // MB

    assert_eq!(config.memory_limit, Some(512 * 1024 * 1024)); // Bytes
}

#[test]
fn test_cpu_shares() {
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_cpu_shares(512);

    assert_eq!(config.cpu_shares, Some(512));
}
```

#### Test: Container Name Generation

```rust
#[test]
fn test_generated_container_name() {
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_version("16".to_string());

    // Name should be like "postgres-16-<random>"
    let name = generate_container_name(&config);
    assert!(name.starts_with("postgres-16-"));
    assert_eq!(name.len(), "postgres-16-XXXXXX".len());
}

#[test]
fn test_custom_container_name() {
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("my-custom-db".to_string());

    assert_eq!(config.name, Some("my-custom-db".to_string()));
}
```

#### Test: Connection String Generation

```rust
#[test]
fn test_postgres_connection_string() {
    let info = ContainerInfo {
        id: "abc123".to_string(),
        name: "test-db".to_string(),
        database_type: "postgres".to_string(),
        version: "16".to_string(),
        port: 5432,
        status: "running".to_string(),
    };

    let conn_str = generate_connection_string(&info);
    assert_eq!(conn_str, "postgres://postgres:postgres@localhost:5432/testdb");
}

#[test]
fn test_mysql_connection_string() {
    let info = ContainerInfo {
        id: "abc123".to_string(),
        name: "test-db".to_string(),
        database_type: "mysql".to_string(),
        version: "8.0".to_string(),
        port: 3306,
        status: "running".to_string(),
    };

    let conn_str = generate_connection_string(&info);
    assert_eq!(conn_str, "mysql://root:mysql@localhost:3306/testdb");
}

#[test]
fn test_sqlserver_connection_string() {
    let info = ContainerInfo {
        id: "abc123".to_string(),
        name: "test-db".to_string(),
        database_type: "sqlserver".to_string(),
        version: "2022-latest".to_string(),
        port: 1433,
        status: "running".to_string(),
    };

    let conn_str = generate_connection_string(&info);
    assert!(conn_str.contains("Server=localhost,1433"));
    assert!(conn_str.contains("User Id=sa"));
}
```

### 1.1 Configuration Module Tests (v0.2.0)

**File**: `tests/unit/config_tests.rs`

#### Test: Config File Parsing

```rust
#[test]
fn test_parse_valid_toml_config() {
    let toml = r#"
        [defaults]
        persistent = false
        memory_mb = 512

        [databases.postgres.env]
        POSTGRES_DB = "testdb"
        POSTGRES_PASSWORD = "secret"
    "#;

    let config = load_config_from_string(toml, ConfigFormat::Toml).unwrap();

    assert_eq!(config.defaults.as_ref().unwrap().memory_mb, Some(512));
    assert_eq!(
        config.databases.get("postgres").unwrap().env.as_ref().unwrap().get("POSTGRES_DB"),
        Some(&"testdb".to_string())
    );
}

#[test]
fn test_parse_invalid_toml_syntax() {
    let toml = r#"
        [defaults
        memory_mb = 512
    "#;

    let result = load_config_from_string(toml, ConfigFormat::Toml);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("TOML"));
}

#[test]
fn test_parse_yaml_config() {
    let yaml = r#"
defaults:
  persistent: false
  memory_mb: 512

databases:
  postgres:
    env:
      POSTGRES_DB: testdb
      POSTGRES_PASSWORD: secret
    "#;

    let config = load_config_from_string(yaml, ConfigFormat::Yaml).unwrap();
    assert_eq!(config.defaults.as_ref().unwrap().memory_mb, Some(512));
}

#[test]
fn test_invalid_memory_value() {
    let toml = r#"
        [defaults]
        memory_mb = -512
    "#;

    let result = load_config_from_string(toml, ConfigFormat::Toml);
    // Should fail validation
    assert!(result.is_err());
}
```

#### Test: Config File Discovery

```rust
#[test]
fn test_find_project_local_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("dbarena.toml");
    std::fs::write(&config_path, "[defaults]\npersistent = true").unwrap();

    // Change to temp dir
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let found = find_config_file().unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap(), config_path);
}

#[test]
fn test_config_precedence() {
    // Test that CLI config overrides project local
    // Test that project local overrides user config
    // Test that user config overrides defaults
}
```

#### Test: Profile Resolution

```rust
#[test]
fn test_resolve_global_profile() {
    let config = DBArenaConfig {
        profiles: HashMap::from([(
            "dev".to_string(),
            ProfileConfig {
                env: HashMap::from([
                    ("LOG_LEVEL".to_string(), "debug".to_string()),
                ]),
            },
        )]),
        ..Default::default()
    };

    let env = resolve_profile(&config, "dev", DatabaseType::Postgres).unwrap();
    assert_eq!(env.get("LOG_LEVEL"), Some(&"debug".to_string()));
}

#[test]
fn test_resolve_database_specific_profile() {
    let config = DBArenaConfig {
        databases: HashMap::from([(
            "postgres".to_string(),
            DatabaseConfig {
                profiles: Some(HashMap::from([(
                    "dev".to_string(),
                    ProfileConfig {
                        env: HashMap::from([
                            ("POSTGRES_DB".to_string(), "myapp_dev".to_string()),
                        ]),
                    },
                )])),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };

    let env = resolve_profile(&config, "dev", DatabaseType::Postgres).unwrap();
    assert_eq!(env.get("POSTGRES_DB"), Some(&"myapp_dev".to_string()));
}

#[test]
fn test_database_profile_overrides_global() {
    let config = DBArenaConfig {
        profiles: HashMap::from([(
            "dev".to_string(),
            ProfileConfig {
                env: HashMap::from([
                    ("LOG_LEVEL".to_string(), "debug".to_string()),
                    ("DB_NAME".to_string(), "global".to_string()),
                ]),
            },
        )]),
        databases: HashMap::from([(
            "postgres".to_string(),
            DatabaseConfig {
                profiles: Some(HashMap::from([(
                    "dev".to_string(),
                    ProfileConfig {
                        env: HashMap::from([
                            ("DB_NAME".to_string(), "postgres_specific".to_string()),
                        ]),
                    },
                )])),
                ..Default::default()
            },
        )]),
        ..Default::default()
    };

    let env = resolve_profile(&config, "dev", DatabaseType::Postgres).unwrap();
    assert_eq!(env.get("LOG_LEVEL"), Some(&"debug".to_string()));
    assert_eq!(env.get("DB_NAME"), Some(&"postgres_specific".to_string()));
}

#[test]
fn test_profile_not_found() {
    let config = DBArenaConfig::default();
    let result = resolve_profile(&config, "nonexistent", DatabaseType::Postgres);

    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Profile not found"));
}

#[test]
fn test_profile_suggestion_levenshtein() {
    let config = DBArenaConfig {
        profiles: HashMap::from([
            ("dev".to_string(), ProfileConfig { env: HashMap::new() }),
            ("test".to_string(), ProfileConfig { env: HashMap::new() }),
            ("prod".to_string(), ProfileConfig { env: HashMap::new() }),
        ]),
        ..Default::default()
    };

    let result = resolve_profile(&config, "dve", DatabaseType::Postgres);
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    // Should suggest "dev" (Levenshtein distance of 1)
    assert!(error.contains("dev"));
}
```

#### Test: Environment Variable Merging

```rust
#[test]
fn test_env_var_precedence() {
    // Create config with defaults
    let base_env = HashMap::from([
        ("POSTGRES_DB".to_string(), "default".to_string()),
        ("POSTGRES_USER".to_string(), "postgres".to_string()),
    ]);

    // Profile env
    let profile_env = HashMap::from([
        ("POSTGRES_DB".to_string(), "profile_db".to_string()),
    ]);

    // CLI env
    let cli_env = HashMap::from([
        ("POSTGRES_DB".to_string(), "cli_db".to_string()),
    ]);

    // Merge: base < profile < CLI
    let mut result = base_env.clone();
    result.extend(profile_env);
    result.extend(cli_env);

    assert_eq!(result.get("POSTGRES_DB"), Some(&"cli_db".to_string()));
    assert_eq!(result.get("POSTGRES_USER"), Some(&"postgres".to_string()));
}

#[test]
fn test_parse_env_args() {
    let args = vec![
        "KEY1=value1".to_string(),
        "KEY2=value2".to_string(),
    ];

    let env = parse_env_args(&args).unwrap();
    assert_eq!(env.get("KEY1"), Some(&"value1".to_string()));
    assert_eq!(env.get("KEY2"), Some(&"value2".to_string()));
}

#[test]
fn test_parse_env_args_invalid() {
    let args = vec!["INVALID".to_string()];
    let result = parse_env_args(&args);
    assert!(result.is_err());
}

#[test]
fn test_parse_env_file() {
    let temp_dir = tempdir().unwrap();
    let env_file = temp_dir.path().join(".env");

    std::fs::write(&env_file, "KEY1=value1\nKEY2=value2\n# Comment\nKEY3=value3").unwrap();

    let env = parse_env_file(env_file).unwrap();
    assert_eq!(env.get("KEY1"), Some(&"value1".to_string()));
    assert_eq!(env.get("KEY2"), Some(&"value2".to_string()));
    assert_eq!(env.get("KEY3"), Some(&"value3".to_string()));
}
```

#### Test: Config Validation

```rust
#[test]
fn test_validate_init_scripts_exist() {
    let temp_dir = tempdir().unwrap();
    let script_path = temp_dir.path().join("schema.sql");
    std::fs::write(&script_path, "CREATE TABLE test;").unwrap();

    let config = DatabaseConfig {
        init_scripts: Some(vec![InitScript::Simple(script_path.to_string_lossy().to_string())]),
        ..Default::default()
    };

    assert!(validate_database_config(&config).is_ok());
}

#[test]
fn test_validate_init_scripts_not_found() {
    let config = DatabaseConfig {
        init_scripts: Some(vec![InitScript::Simple("/nonexistent/script.sql".to_string())]),
        ..Default::default()
    };

    let result = validate_database_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_validate_env_var_names() {
    // Invalid: spaces in key
    let env = HashMap::from([("KEY WITH SPACES".to_string(), "value".to_string())]);
    assert!(validate_env_vars(&env).is_err());

    // Valid
    let env = HashMap::from([("VALID_KEY".to_string(), "value".to_string())]);
    assert!(validate_env_vars(&env).is_ok());
}
```

### 1.2 Init Script Module Tests

**File**: `tests/unit/init_tests.rs`

#### Test: Error Parsing

```rust
#[test]
fn test_postgres_error_parser() {
    let output = r#"
ERROR:  syntax error at or near "INSRT"
LINE 15: INSRT INTO users (name, email) VALUES ('Alice', 'alice@example.com');
         ^
    "#;

    let parser = PostgresErrorParser;
    let error = parser.parse_error(output, "script content").unwrap();

    assert_eq!(error.line_number, Some(15));
    assert!(error.error_message.contains("syntax error"));
    assert!(error.suggestion.is_some());
}

#[test]
fn test_postgres_typo_suggestion() {
    let output = "ERROR: syntax error at or near \"INSRT\"";
    let parser = PostgresErrorParser;
    let error = parser.parse_error(output, "INSRT INTO users").unwrap();

    assert!(error.suggestion.as_ref().unwrap().contains("INSERT"));
}

#[test]
fn test_mysql_error_parser() {
    let output = r#"
ERROR 1064 (42000) at line 10: You have an error in your SQL syntax
    "#;

    let parser = MySQLErrorParser;
    let error = parser.parse_error(output, "script content").unwrap();

    assert_eq!(error.line_number, Some(10));
    assert_eq!(error.database_error_code, Some("1064".to_string()));
}

#[test]
fn test_sqlserver_error_parser() {
    let output = r#"
Msg 156, Level 15, State 1, Server localhost, Line 20
Incorrect syntax near the keyword 'FROM'.
    "#;

    let parser = SQLServerErrorParser;
    let error = parser.parse_error(output, "script content").unwrap();

    assert_eq!(error.line_number, Some(20));
    assert_eq!(error.database_error_code, Some("156".to_string()));
}
```

#### Test: Log Management

```rust
#[test]
fn test_create_log_session() {
    let temp_dir = tempdir().unwrap();
    let log_manager = LogManager::new(Some(temp_dir.path().to_path_buf())).unwrap();

    let session = log_manager.create_session("test-container-123").unwrap();

    assert!(session.session_dir.exists());
    assert_eq!(session.container_id, "test-container-123");
}

#[test]
fn test_write_script_log() {
    let temp_dir = tempdir().unwrap();
    let log_manager = LogManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
    let session = log_manager.create_session("test-container").unwrap();

    let script = PathBuf::from("schema.sql");
    let output = "CREATE TABLE users;\nCREATE TABLE posts;";

    let log_path = log_manager.write_script_log(&session, &script, output).unwrap();

    assert!(log_path.exists());
    let content = std::fs::read_to_string(&log_path).unwrap();
    assert!(content.contains("CREATE TABLE"));
}

#[test]
fn test_write_metadata() {
    let temp_dir = tempdir().unwrap();
    let log_manager = LogManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
    let session = log_manager.create_session("test-container").unwrap();

    let metadata = ExecutionMetadata {
        scripts: vec![ScriptMetadata {
            path: PathBuf::from("test.sql"),
            success: true,
            duration: Duration::from_secs(1),
            log_file: PathBuf::from("test.sql.log"),
            error_summary: None,
        }],
        total_duration: Duration::from_secs(1),
        success_count: 1,
        failure_count: 0,
    };

    log_manager.write_metadata(&session, &metadata).unwrap();

    let metadata_path = session.session_dir.join("metadata.json");
    assert!(metadata_path.exists());
}

#[test]
fn test_get_session_logs() {
    let temp_dir = tempdir().unwrap();
    let log_manager = LogManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
    let session = log_manager.create_session("test-container").unwrap();

    log_manager.write_script_log(&session, &PathBuf::from("test.sql"), "output").unwrap();

    let logs = log_manager.get_session_logs("test-container").unwrap();
    assert!(!logs.is_empty());
}
```

### 1.3 Container Config Tests

**File**: `tests/unit/container_tests.rs`

```rust
#[test]
fn test_container_config_builder() {
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_version("16".to_string())
        .with_name("test-db".to_string())
        .with_port(5433)
        .with_env_vars(HashMap::from([
            ("POSTGRES_DB".to_string(), "custom".to_string()),
        ]))
        .with_init_scripts(vec![PathBuf::from("schema.sql")])
        .with_continue_on_error(true);

    assert_eq!(config.version, "16");
    assert_eq!(config.name, Some("test-db".to_string()));
    assert_eq!(config.port, Some(5433));
    assert_eq!(config.env_vars.get("POSTGRES_DB"), Some(&"custom".to_string()));
    assert_eq!(config.init_scripts.len(), 1);
    assert!(config.continue_on_error);
}

#[test]
fn test_build_env_vars_with_custom() {
    // Test that custom env vars override defaults
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_env_vars(HashMap::from([
            ("POSTGRES_DB".to_string(), "custom_db".to_string()),
        ]));

    let env_vars = build_env_vars(&config);

    // Should have default + custom
    assert!(env_vars.iter().any(|e| e == "POSTGRES_DB=custom_db"));
    assert!(env_vars.iter().any(|e| e.starts_with("POSTGRES_PASSWORD=")));
}
```

---

## Phase 2: Integration Tests

### 2.1 End-to-End Config Loading

**File**: `tests/integration/config_integration_tests.rs`

**Requires**: Docker daemon running

```rust
#[tokio::test]
#[ignore] // Run with: cargo test --test integration -- --ignored
async fn test_create_with_config_file() {
    // Setup: Create temp config
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("dbarena.toml");

    std::fs::write(&config_file, r#"
[databases.postgres.env]
POSTGRES_DB = "integration_test"
POSTGRES_PASSWORD = "test123"
    "#).unwrap();

    // Execute: Create container
    let result = create_container_with_config(
        DatabaseType::Postgres,
        Some(config_file),
        None, // No profile
    ).await;

    assert!(result.is_ok());
    let container = result.unwrap();

    // Verify: Check env vars
    let inspect = docker_client.inspect_container(&container.id).await.unwrap();
    let env = inspect.config.unwrap().env.unwrap();

    assert!(env.iter().any(|e| e == "POSTGRES_DB=integration_test"));
    assert!(env.iter().any(|e| e == "POSTGRES_PASSWORD=test123"));

    // Cleanup
    docker_client.remove_container(&container.id, true).await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_create_with_profile() {
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("dbarena.toml");

    std::fs::write(&config_file, r#"
[profiles.test]
env = { LOG_LEVEL = "debug" }

[databases.postgres.profiles.test]
env = { POSTGRES_DB = "test_db" }
    "#).unwrap();

    let result = create_container_with_config(
        DatabaseType::Postgres,
        Some(config_file),
        Some("test".to_string()),
    ).await;

    assert!(result.is_ok());
    let container = result.unwrap();

    // Verify env vars include profile settings
    let inspect = docker_client.inspect_container(&container.id).await.unwrap();
    let env = inspect.config.unwrap().env.unwrap();

    assert!(env.iter().any(|e| e == "POSTGRES_DB=test_db"));
    assert!(env.iter().any(|e| e == "LOG_LEVEL=debug"));

    // Cleanup
    docker_client.remove_container(&container.id, true).await.unwrap();
}
```

### 2.2 Init Script Execution

**File**: `tests/integration/init_script_tests.rs`

```rust
#[tokio::test]
#[ignore]
async fn test_postgres_init_script_success() {
    let temp_dir = tempdir().unwrap();
    let script = temp_dir.path().join("schema.sql");

    std::fs::write(&script, r#"
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100)
);

INSERT INTO users (name) VALUES ('Alice'), ('Bob');
    "#).unwrap();

    // Create container
    let container = create_test_container(DatabaseType::Postgres).await.unwrap();
    wait_for_healthy(&container.id).await.unwrap();

    // Execute init script
    let log_manager = LogManager::new(None).unwrap();
    let results = execute_init_scripts(
        docker.docker(),
        &container.id,
        vec![script.clone()],
        DatabaseType::Postgres,
        &container.config,
        false,
        &log_manager,
    ).await.unwrap();

    // Verify success
    assert_eq!(results.len(), 1);
    assert!(results[0].success);
    assert!(results[0].statements_executed >= 2);

    // Verify data was inserted
    let query_result = execute_query(
        &container.id,
        "SELECT COUNT(*) FROM users;",
    ).await.unwrap();
    assert!(query_result.contains("2"));

    // Cleanup
    cleanup_container(&container.id).await;
}

#[tokio::test]
#[ignore]
async fn test_init_script_failure_stops_creation() {
    let temp_dir = tempdir().unwrap();
    let bad_script = temp_dir.path().join("bad.sql");

    std::fs::write(&bad_script, "INSRT INTO users VALUES (1);").unwrap();

    let container = create_test_container(DatabaseType::Postgres).await.unwrap();
    wait_for_healthy(&container.id).await.unwrap();

    let log_manager = LogManager::new(None).unwrap();
    let results = execute_init_scripts(
        docker.docker(),
        &container.id,
        vec![bad_script],
        DatabaseType::Postgres,
        &container.config,
        false, // Don't continue on error
        &log_manager,
    ).await;

    // Should fail
    assert!(results.is_ok()); // Returns results, not error
    let results = results.unwrap();
    assert_eq!(results.len(), 1);
    assert!(!results[0].success);
    assert!(results[0].error.is_some());

    // Verify error message has context
    let error = results[0].error.as_ref().unwrap();
    assert!(error.error_message.contains("syntax error"));
    assert!(error.suggestion.is_some());

    cleanup_container(&container.id).await;
}

#[tokio::test]
#[ignore]
async fn test_init_script_continue_on_error() {
    let temp_dir = tempdir().unwrap();

    let script1 = temp_dir.path().join("good.sql");
    std::fs::write(&script1, "CREATE TABLE test1 (id INT);").unwrap();

    let script2 = temp_dir.path().join("bad.sql");
    std::fs::write(&script2, "INVALID SQL;").unwrap();

    let script3 = temp_dir.path().join("good2.sql");
    std::fs::write(&script3, "CREATE TABLE test2 (id INT);").unwrap();

    let container = create_test_container(DatabaseType::Postgres).await.unwrap();
    wait_for_healthy(&container.id).await.unwrap();

    let log_manager = LogManager::new(None).unwrap();
    let results = execute_init_scripts(
        docker.docker(),
        &container.id,
        vec![script1, script2, script3],
        DatabaseType::Postgres,
        &container.config,
        true, // Continue on error
        &log_manager,
    ).await.unwrap();

    // All scripts attempted
    assert_eq!(results.len(), 3);
    assert!(results[0].success);
    assert!(!results[1].success);
    assert!(results[2].success);

    // Verify both good tables exist
    let query = execute_query(&container.id, "SELECT COUNT(*) FROM test1;").await;
    assert!(query.is_ok());

    let query = execute_query(&container.id, "SELECT COUNT(*) FROM test2;").await;
    assert!(query.is_ok());

    cleanup_container(&container.id).await;
}

#[tokio::test]
#[ignore]
async fn test_mysql_init_script() {
    let temp_dir = tempdir().unwrap();
    let script = temp_dir.path().join("schema.sql");

    std::fs::write(&script, r#"
CREATE TABLE IF NOT EXISTS products (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(100)
);

INSERT INTO products (name) VALUES ('Product A'), ('Product B');
    "#).unwrap();

    let container = create_test_container(DatabaseType::MySQL).await.unwrap();
    wait_for_healthy(&container.id).await.unwrap();

    let log_manager = LogManager::new(None).unwrap();
    let results = execute_init_scripts(
        docker.docker(),
        &container.id,
        vec![script],
        DatabaseType::MySQL,
        &container.config,
        false,
        &log_manager,
    ).await.unwrap();

    assert!(results[0].success);

    cleanup_container(&container.id).await;
}

#[tokio::test]
#[ignore]
async fn test_sqlserver_init_script() {
    let temp_dir = tempdir().unwrap();
    let script = temp_dir.path().join("schema.sql");

    std::fs::write(&script, r#"
CREATE TABLE orders (
    id INT PRIMARY KEY,
    total DECIMAL(10,2)
);
GO

INSERT INTO orders VALUES (1, 99.99);
GO
    "#).unwrap();

    let container = create_test_container(DatabaseType::SQLServer).await.unwrap();
    wait_for_healthy(&container.id).await.unwrap();

    let log_manager = LogManager::new(None).unwrap();
    let results = execute_init_scripts(
        docker.docker(),
        &container.id,
        vec![script],
        DatabaseType::SQLServer,
        &container.config,
        false,
        &log_manager,
    ).await.unwrap();

    assert!(results[0].success);

    cleanup_container(&container.id).await;
}
```

### 2.3 Exec Command Tests

**File**: `tests/integration/exec_tests.rs`

```rust
#[tokio::test]
#[ignore]
async fn test_exec_inline_sql() {
    // Create and start container
    let container = create_test_container(DatabaseType::Postgres).await.unwrap();
    wait_for_healthy(&container.id).await.unwrap();

    // Execute inline SQL
    let result = handle_exec(
        Some(container.name.clone()),
        false,
        Some("SELECT 1 as test;".to_string()),
        None,
    ).await;

    assert!(result.is_ok());

    cleanup_container(&container.id).await;
}

#[tokio::test]
#[ignore]
async fn test_exec_from_file() {
    let temp_dir = tempdir().unwrap();
    let query_file = temp_dir.path().join("query.sql");
    std::fs::write(&query_file, "SELECT current_database();").unwrap();

    let container = create_test_container(DatabaseType::Postgres).await.unwrap();
    wait_for_healthy(&container.id).await.unwrap();

    let result = handle_exec(
        Some(container.name.clone()),
        false,
        None,
        Some(query_file),
    ).await;

    assert!(result.is_ok());

    cleanup_container(&container.id).await;
}

#[tokio::test]
#[ignore]
async fn test_exec_with_error() {
    let container = create_test_container(DatabaseType::Postgres).await.unwrap();
    wait_for_healthy(&container.id).await.unwrap();

    let result = handle_exec(
        Some(container.name.clone()),
        false,
        Some("INVALID SQL SYNTAX;".to_string()),
        None,
    ).await;

    // Should return error
    assert!(result.is_err());

    cleanup_container(&container.id).await;
}
```

### 2.4 Config Utility Commands

**File**: `tests/integration/config_commands_tests.rs`

```rust
#[test]
fn test_config_validate_valid() {
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("dbarena.toml");

    std::fs::write(&config_file, r#"
[defaults]
persistent = false

[databases.postgres.env]
POSTGRES_DB = "test"
    "#).unwrap();

    let result = handle_config_validate(Some(config_file), false);
    assert!(result.is_ok());
}

#[test]
fn test_config_validate_invalid_syntax() {
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("bad.toml");

    std::fs::write(&config_file, "[defaults\npersistent = false").unwrap();

    let result = handle_config_validate(Some(config_file), false);
    assert!(result.is_err());
}

#[test]
fn test_config_show() {
    let temp_dir = tempdir().unwrap();
    let config_file = temp_dir.path().join("dbarena.toml");

    std::fs::write(&config_file, r#"
[profiles.dev]
env = { LOG_LEVEL = "debug" }
    "#).unwrap();

    let result = handle_config_show(Some(config_file), Some("dev".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_config_init() {
    let temp_dir = tempdir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = handle_config_init();
    assert!(result.is_ok());

    let config_file = temp_dir.path().join("dbarena.toml");
    assert!(config_file.exists());
}
```

### 2.5 v0.1.0 Integration Tests

**File**: `tests/integration/v0_1_0_integration_tests.rs`

#### Test: MySQL Health Checking

```rust
#[tokio::test]
#[ignore]
async fn test_mysql_health_check() {
    let docker_client = DockerClient::new().unwrap();
    docker_client.verify_connection().await.unwrap();

    let manager = ContainerManager::new(docker_client);

    let config = ContainerConfig::new(DatabaseType::MySQL)
        .with_name("test-mysql-health".to_string());

    let container = manager.create_container(config).await.unwrap();
    manager.start_container(&container.id).await.unwrap();

    // Test health check
    let checker = MySQLHealthChecker::new(DockerClient::new().unwrap().docker().clone());
    let result = wait_for_healthy(&container.id, &checker, Duration::from_secs(60)).await;

    assert!(result.is_ok());

    cleanup_container(&container.id).await;
}
```

#### Test: SQL Server Health Checking

```rust
#[tokio::test]
#[ignore]
async fn test_sqlserver_health_check() {
    let docker_client = DockerClient::new().unwrap();
    docker_client.verify_connection().await.unwrap();

    let manager = ContainerManager::new(docker_client);

    let config = ContainerConfig::new(DatabaseType::SQLServer)
        .with_name("test-sqlserver-health".to_string());

    let container = manager.create_container(config).await.unwrap();
    manager.start_container(&container.id).await.unwrap();

    // Test health check
    let checker = SQLServerHealthChecker::new(DockerClient::new().unwrap().docker().clone());
    let result = wait_for_healthy(&container.id, &checker, Duration::from_secs(120)).await;

    assert!(result.is_ok());

    cleanup_container(&container.id).await;
}
```

#### Test: Restart Container

```rust
#[tokio::test]
#[ignore]
async fn test_restart_container() {
    let docker_client = DockerClient::new().unwrap();
    let manager = ContainerManager::new(docker_client);

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("test-restart".to_string());

    let container = manager.create_container(config).await.unwrap();
    manager.start_container(&container.id).await.unwrap();

    // Restart
    let result = manager.restart_container(&container.id, Some(5)).await;
    assert!(result.is_ok());

    // Verify still running
    let info = manager.find_container(&container.id).await.unwrap().unwrap();
    assert_eq!(info.status, "running");

    cleanup_container(&container.id).await;
}
```

#### Test: Inspect Container

```rust
#[tokio::test]
#[ignore]
async fn test_inspect_container() {
    let docker_client = DockerClient::new().unwrap();
    let manager = ContainerManager::new(docker_client);

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("test-inspect".to_string())
        .with_memory_limit(512)
        .with_cpu_shares(512);

    let container = manager.create_container(config).await.unwrap();

    // Inspect
    let details = manager.inspect_container(&container.id).await.unwrap();

    // Verify memory limit
    assert_eq!(details.host_config.memory, Some(512 * 1024 * 1024));
    // Verify CPU shares
    assert_eq!(details.host_config.cpu_shares, Some(512));

    cleanup_container(&container.id).await;
}
```

#### Test: Container Logs

```rust
#[tokio::test]
#[ignore]
async fn test_get_container_logs() {
    let docker_client = DockerClient::new().unwrap();
    let manager = ContainerManager::new(docker_client);

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("test-logs".to_string());

    let container = manager.create_container(config).await.unwrap();
    manager.start_container(&container.id).await.unwrap();

    // Wait a bit for logs to generate
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Get logs
    let logs = manager.get_logs(&container.id, None).await.unwrap();

    // Should have some output
    assert!(!logs.is_empty());
    assert!(logs.contains("PostgreSQL") || logs.contains("database system is ready"));

    cleanup_container(&container.id).await;
}
```

#### Test: Port Auto-Assignment

```rust
#[tokio::test]
#[ignore]
async fn test_port_auto_assignment() {
    let docker_client = DockerClient::new().unwrap();
    let manager = ContainerManager::new(docker_client);

    // Create without specifying port
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("test-auto-port".to_string());

    let container = manager.create_container(config).await.unwrap();

    // Should have assigned a port
    assert!(container.port > 0);
    assert!(container.port != 5432); // Should be random, not default

    cleanup_container(&container.id).await;
}
```

#### Test: Custom Port Assignment

```rust
#[tokio::test]
#[ignore]
async fn test_custom_port_assignment() {
    let docker_client = DockerClient::new().unwrap();
    let manager = ContainerManager::new(docker_client);

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("test-custom-port".to_string())
        .with_port(5433);

    let container = manager.create_container(config).await.unwrap();

    // Should use specified port
    assert_eq!(container.port, 5433);

    cleanup_container(&container.id).await;
}
```

#### Test: Port Conflict Handling

```rust
#[tokio::test]
#[ignore]
async fn test_port_conflict() {
    let docker_client = DockerClient::new().unwrap();
    let manager = ContainerManager::new(docker_client);

    // Create first container on port 5433
    let config1 = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("test-port-1".to_string())
        .with_port(5433);

    let container1 = manager.create_container(config1).await.unwrap();
    manager.start_container(&container1.id).await.unwrap();

    // Try to create second container on same port
    let config2 = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("test-port-2".to_string())
        .with_port(5433);

    let result = manager.create_container(config2).await;

    // Should fail or assign different port
    assert!(result.is_err() || result.unwrap().port != 5433);

    cleanup_container(&container1.id).await;
}
```

#### Test: Container Labels

```rust
#[tokio::test]
#[ignore]
async fn test_dbarena_labels() {
    let docker_client = DockerClient::new().unwrap();
    let manager = ContainerManager::new(docker_client);

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("test-labels".to_string())
        .with_version("16".to_string());

    let container = manager.create_container(config).await.unwrap();

    // Inspect to check labels
    let details = manager.inspect_container(&container.id).await.unwrap();
    let labels = details.config.labels;

    assert_eq!(labels.get("dbarena.managed"), Some(&"true".to_string()));
    assert_eq!(labels.get("dbarena.database_type"), Some(&"postgres".to_string()));
    assert_eq!(labels.get("dbarena.version"), Some(&"16".to_string()));

    cleanup_container(&container.id).await;
}
```

#### Test: Multi-Version Support

```rust
#[tokio::test]
#[ignore]
async fn test_multi_version_postgres() {
    let docker_client = DockerClient::new().unwrap();
    let manager = ContainerManager::new(docker_client);

    // Create PostgreSQL 16
    let config16 = ContainerConfig::new(DatabaseType::Postgres)
        .with_version("16".to_string())
        .with_name("test-pg-16".to_string());

    // Create PostgreSQL 15
    let config15 = ContainerConfig::new(DatabaseType::Postgres)
        .with_version("15".to_string())
        .with_name("test-pg-15".to_string());

    let container16 = manager.create_container(config16).await.unwrap();
    let container15 = manager.create_container(config15).await.unwrap();

    // Both should be created with different names and ports
    assert_ne!(container16.id, container15.id);
    assert_ne!(container16.port, container15.port);
    assert!(container16.name.contains("16"));
    assert!(container15.name.contains("15"));

    cleanup_container(&container16.id).await;
    cleanup_container(&container15.id).await;
}
```

#### Test: Resource Constraints Applied

```rust
#[tokio::test]
#[ignore]
async fn test_resource_constraints() {
    let docker_client = DockerClient::new().unwrap();
    let manager = ContainerManager::new(docker_client);

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("test-resources".to_string())
        .with_memory_limit(512)
        .with_cpu_shares(512);

    let container = manager.create_container(config).await.unwrap();

    // Inspect to verify constraints
    let inspect = manager.inspect_container(&container.id).await.unwrap();

    assert_eq!(inspect.host_config.memory, Some(512 * 1024 * 1024));
    assert_eq!(inspect.host_config.cpu_shares, Some(512));

    cleanup_container(&container.id).await;
}
```

#### Test: Persistent Storage

```rust
#[tokio::test]
#[ignore]
async fn test_persistent_storage() {
    let docker_client = DockerClient::new().unwrap();
    let manager = ContainerManager::new(docker_client);

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("test-persistent".to_string())
        .with_persistent(true);

    let container = manager.create_container(config).await.unwrap();

    // Check for volume mounts
    let inspect = manager.inspect_container(&container.id).await.unwrap();

    assert!(!inspect.mounts.is_empty());
    assert!(inspect.mounts.iter().any(|m| m.mount_type == "volume"));

    // Cleanup with volume removal
    manager.destroy_container(&container.id, true).await.unwrap();
}
```

#### Test: Batch Destroy

```rust
#[tokio::test]
#[ignore]
async fn test_batch_destroy() {
    let docker_client = DockerClient::new().unwrap();
    let manager = ContainerManager::new(docker_client);

    // Create multiple containers
    let mut containers = vec![];
    for i in 0..3 {
        let config = ContainerConfig::new(DatabaseType::Postgres)
            .with_name(format!("test-batch-{}", i));

        let container = manager.create_container(config).await.unwrap();
        containers.push(container.id);
    }

    // Destroy all at once
    for id in &containers {
        manager.destroy_container(id, false).await.unwrap();
    }

    // Verify all gone
    let remaining = manager.list_containers(true).await.unwrap();
    for id in containers {
        assert!(!remaining.iter().any(|c| c.id == id));
    }
}
```

---

## Phase 3: Manual Testing

### 3.0 v0.1.0 Core Functionality

#### Test Case 0.1: Basic Container Creation (Postgres)

**Steps:**
1. Run: `dbarena create postgres`
2. Wait for completion
3. Observe output

**Expected:**
- Container created with default port (auto-assigned)
- Health check passes
- Connection string displayed
- Container name includes "postgres-16-<random>"

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.2: Basic Container Creation (MySQL)

**Steps:**
1. Run: `dbarena create mysql`
2. Wait for completion

**Expected:**
- Container created successfully
- Health check passes
- Connection string includes mysql://root:mysql@localhost:<port>/testdb

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.3: Basic Container Creation (SQL Server)

**Steps:**
1. Run: `dbarena create sqlserver`
2. Wait for completion (may take longer)

**Expected:**
- Container created successfully
- Health check passes (may take 30-60s)
- Connection string displayed

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.4: List Containers

**Steps:**
1. Create 2-3 containers
2. Run: `dbarena list`
3. Verify output

**Expected:**
- Table showing all containers
- Columns: ID, Name, Database, Version, Port, Status
- Only running containers shown by default

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.5: List All Containers (Including Stopped)

**Steps:**
1. Create container, then stop it
2. Run: `dbarena list --all`

**Expected:**
- Shows both running and stopped containers
- Status column shows "stopped" for stopped container

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.6: Stop Container

**Steps:**
1. Create running container
2. Run: `dbarena stop <container-name>`
3. Verify with `dbarena list --all`

**Expected:**
- Container stops gracefully
- Status changes to "stopped"
- Success message displayed

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.7: Start Stopped Container

**Steps:**
1. Stop a container
2. Run: `dbarena start <container-name>`
3. Wait for health check

**Expected:**
- Container starts
- Health check passes again
- Status becomes "running"

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.8: Restart Running Container

**Steps:**
1. Create running container
2. Run: `dbarena restart <container-name>`
3. Wait for completion

**Expected:**
- Container restarts gracefully
- Health check passes again
- Same port maintained

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.9: Inspect Container

**Steps:**
1. Create container with: `dbarena create postgres --memory 512 --cpu-shares 512`
2. Run: `dbarena inspect <container-name>`
3. Review output

**Expected:**
- Detailed container information displayed
- Memory limit: 512 MB
- CPU shares: 512
- Environment variables shown
- Port mappings shown

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.10: View Container Logs

**Steps:**
1. Create running container
2. Run: `dbarena logs <container-name>`

**Expected:**
- Database startup logs displayed
- Shows initialization messages
- Readable format

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.11: Follow Container Logs

**Steps:**
1. Create running container
2. Run: `dbarena logs <container-name> --follow`
3. Watch for a few seconds
4. Press Ctrl+C

**Expected:**
- Logs stream in real-time
- New log lines appear as they're generated
- Clean exit on Ctrl+C

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.12: Destroy Container

**Steps:**
1. Create container
2. Run: `dbarena destroy <container-name>`
3. Confirm when prompted

**Expected:**
- Confirmation prompt appears
- After confirmation, container destroyed
- Success message shown
- Container no longer in `dbarena list --all`

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.13: Destroy Without Confirmation

**Steps:**
1. Create container
2. Run: `dbarena destroy <container-name> -y`

**Expected:**
- No confirmation prompt
- Container destroyed immediately
- Success message

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.14: Destroy with Volume Removal

**Steps:**
1. Create persistent container: `dbarena create postgres --persistent --name test-persistent`
2. Run: `dbarena destroy test-persistent -v`
3. Confirm

**Expected:**
- Container and associated volumes removed
- Success message mentions volume removal

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.15: Custom Container Name

**Steps:**
1. Run: `dbarena create postgres --name my-custom-db`

**Expected:**
- Container created with exact name "my-custom-db"
- No random suffix added

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.16: Custom Port

**Steps:**
1. Run: `dbarena create postgres --port 5433`
2. Check output

**Expected:**
- Container uses port 5433
- Connection string shows :5433

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.17: Custom Version

**Steps:**
1. Run: `dbarena create postgres --version 15`
2. Check output

**Expected:**
- Container name includes "postgres-15-"
- Uses PostgreSQL 15 image

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.18: Memory Limit

**Steps:**
1. Run: `dbarena create postgres --memory 512 --name test-mem`
2. Run: `dbarena inspect test-mem`
3. Check memory settings

**Expected:**
- Memory limit set to 512 MB
- Visible in inspect output

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.19: CPU Shares

**Steps:**
1. Run: `dbarena create postgres --cpu-shares 512 --name test-cpu`
2. Run: `dbarena inspect test-cpu`

**Expected:**
- CPU shares set to 512
- Visible in inspect output

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.20: Persistent Storage

**Steps:**
1. Run: `dbarena create postgres --persistent --name test-persist`
2. Run: `dbarena inspect test-persist`
3. Check for volume mounts

**Expected:**
- Volume created and mounted
- Data persists across container restarts
- Inspect shows volume mount

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.21: Multiple Database Types

**Steps:**
1. Run: `dbarena create postgres mysql sqlserver`
2. Wait for all to complete

**Expected:**
- All three containers created in parallel
- Each gets unique port
- All health checks pass

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.22: Multi-Version PostgreSQL

**Steps:**
1. Run: `dbarena create postgres --version 16 --name pg16`
2. Run: `dbarena create postgres --version 15 --name pg15`
3. Run: `dbarena create postgres --version 14 --name pg14`
4. Run: `dbarena list`

**Expected:**
- All three versions running simultaneously
- Each has unique port
- All show as "running"

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.23: Port Conflict Handling

**Steps:**
1. Create container on port 5432
2. Try to create another on same port

**Expected:**
- Second creation fails with clear error
- Error message: "Port 5432 is already in use"

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.24: Docker Not Running

**Steps:**
1. Stop Docker daemon
2. Run: `dbarena create postgres`

**Expected:**
- Clear error: "Docker daemon not running or not accessible"
- Helpful message to start Docker

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.25: Main Interactive Menu

**Steps:**
1. Run: `dbarena` (no arguments)
2. Navigate menu

**Expected:**
- Interactive menu appears
- Options: Create, List, Start, Stop, Restart, Destroy, Inspect, Logs, Exit
- Arrow keys work
- Selection works

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.26: Interactive Create

**Steps:**
1. Run: `dbarena create -i`
2. Select PostgreSQL
3. Select version 16
4. Continue through prompts

**Expected:**
- Multi-select database types
- Multi-select versions
- Advanced options (optional)
- Confirmation
- Container created

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.27: Interactive Destroy

**Steps:**
1. Create 3 containers
2. Run: `dbarena destroy -i`
3. Multi-select 2 of them
4. Confirm

**Expected:**
- Shows list of containers
- Can select multiple
- Batch confirmation option
- Selected containers destroyed

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.28: Help Text

**Steps:**
1. Run: `dbarena --help`
2. Run: `dbarena create --help`

**Expected:**
- Comprehensive help displayed
- All commands listed
- All flags explained
- Examples shown

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.29: Version Command

**Steps:**
1. Run: `dbarena --version`

**Expected:**
- Version number displayed (e.g., "dbarena 0.2.0")

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 0.30: Connection String Accuracy

**Steps:**
1. Create postgres container
2. Copy connection string from output
3. Use psql with that connection string

**Expected:**
- Connection string is accurate and works
- Can connect successfully
- Database exists

**Actual:** [ ]

**Issues Found:**

---

### 3.1 Configuration Management (v0.2.0)

#### Test Case 1: Basic Config File Usage

**Steps:**
1. Create `dbarena.toml` in project directory:
   ```toml
   [databases.postgres.env]
   POSTGRES_DB = "myapp"
   POSTGRES_PASSWORD = "secret123"
   ```
2. Run: `dbarena create postgres`
3. Verify container uses custom env vars

**Expected:**
- Container created successfully
- Connection string shows `myapp` database
- Environment variables visible in `dbarena inspect`

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 2: Profile Selection (CLI)

**Steps:**
1. Create config with profiles:
   ```toml
   [databases.postgres.profiles.dev]
   env = { POSTGRES_DB = "myapp_dev" }

   [databases.postgres.profiles.prod]
   env = { POSTGRES_DB = "myapp_prod" }
   ```
2. Run: `dbarena create postgres --profile dev`
3. Verify dev profile is used

**Expected:**
- Container created with `myapp_dev` database
- Profile name logged in verbose mode

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 3: Interactive Profile Selection

**Steps:**
1. Create config with multiple profiles
2. Run: `dbarena create -i`
3. Select PostgreSQL
4. Select version 16
5. Observe profile selection prompt
6. Select `dev` profile
7. Complete creation

**Expected:**
- Profile selection appears after version selection
- Shows list of available profiles
- "No profile" option available
- Selected profile applied to container

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 4: CLI Override Precedence

**Steps:**
1. Create config: `POSTGRES_DB = "config_db"`
2. Run: `dbarena create postgres --env POSTGRES_DB=cli_db`
3. Verify CLI value wins

**Expected:**
- Container uses `cli_db` (CLI override)
- Config value ignored

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 5: Env File Loading

**Steps:**
1. Create `.env.local`:
   ```
   POSTGRES_DB=envfile_db
   POSTGRES_PASSWORD=envfile_pass
   ```
2. Run: `dbarena create postgres --env-file .env.local`
3. Verify env file values used

**Expected:**
- Container uses values from env file
- Other defaults remain

**Actual:** [ ]

**Issues Found:**

---

### 3.2 Initialization Scripts

#### Test Case 6: Single Init Script

**Steps:**
1. Create `schema.sql`:
   ```sql
   CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(100));
   INSERT INTO users (name) VALUES ('Test User');
   ```
2. Run: `dbarena create postgres --init-script ./schema.sql`
3. Wait for completion
4. Connect: `psql -h localhost -p <port> -U postgres -d testdb`
5. Query: `SELECT * FROM users;`

**Expected:**
- Container creates successfully
- "Running initialization scripts..." message shown
- Script executes successfully
- Table exists with 1 row
- Log saved to `~/.local/share/dbarena/logs/`

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 7: Multiple Init Scripts in Order

**Steps:**
1. Create `1-schema.sql`: `CREATE TABLE test (id INT);`
2. Create `2-data.sql`: `INSERT INTO test VALUES (1);`
3. Run: `dbarena create postgres --init-script ./1-schema.sql --init-script ./2-data.sql`
4. Verify execution order

**Expected:**
- Both scripts execute in order
- Progress shown for each script
- Data inserted successfully

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 8: Init Script Failure (Stop Creation)

**Steps:**
1. Create `bad.sql`: `INSRT INTO test VALUES (1);`
2. Run: `dbarena create postgres --init-script ./bad.sql`
3. Observe error handling

**Expected:**
- Container starts
- Health check passes
- Script execution fails
- Clear error message with:
  - Line number
  - Error context
  - Suggestion (INSRT → INSERT)
  - Log file location
- Container destroyed (default behavior)

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 9: Continue on Error

**Steps:**
1. Create `1-good.sql`: `CREATE TABLE test1 (id INT);`
2. Create `2-bad.sql`: `INVALID SQL;`
3. Create `3-good.sql`: `CREATE TABLE test2 (id INT);`
4. Run: `dbarena create postgres --init-script ./1-good.sql --init-script ./2-bad.sql --init-script ./3-good.sql --continue-on-error`

**Expected:**
- Script 1 succeeds
- Script 2 fails with warning
- Script 3 still executes
- Container remains running
- Warning summary shown

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 10: Init Script with MySQL

**Steps:**
1. Create `mysql-schema.sql`:
   ```sql
   CREATE TABLE products (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(100));
   INSERT INTO products (name) VALUES ('Product A');
   ```
2. Run: `dbarena create mysql --init-script ./mysql-schema.sql`
3. Verify execution

**Expected:**
- MySQL-specific execution command used
- Script succeeds
- Table created

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 11: Init Script with SQL Server

**Steps:**
1. Create `sqlserver-schema.sql`:
   ```sql
   CREATE TABLE orders (id INT PRIMARY KEY, total DECIMAL(10,2));
   GO
   INSERT INTO orders VALUES (1, 99.99);
   GO
   ```
2. Run: `dbarena create sqlserver --init-script ./sqlserver-schema.sql`
3. Verify execution

**Expected:**
- SQL Server-specific execution (sqlcmd)
- GO statements handled correctly
- Script succeeds

**Actual:** [ ]

**Issues Found:**

---

### 3.3 SQL Execution (Exec Command)

#### Test Case 12: Exec Inline SQL

**Steps:**
1. Create container: `dbarena create postgres --name test-db`
2. Run: `dbarena exec --container test-db --script "SELECT version();"`
3. Observe output

**Expected:**
- SQL executes successfully
- Version info displayed
- Execution time shown
- No temp file left behind

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 13: Exec from File

**Steps:**
1. Create `query.sql`: `SELECT current_database(), current_user;`
2. Create container
3. Run: `dbarena exec --container test-db --file ./query.sql`

**Expected:**
- Script executes
- Results displayed
- Log saved

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 14: Exec Interactive Mode

**Steps:**
1. Create 2 containers: postgres and mysql
2. Run: `dbarena exec -i --script "SELECT 1;"`
3. Select postgres container from list
4. Observe execution

**Expected:**
- Interactive container list appears
- Shows running containers only
- Selection works
- Script executes on selected container

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 15: Exec with Error

**Steps:**
1. Create container
2. Run: `dbarena exec --container test-db --script "INVALID SQL;"`
3. Observe error handling

**Expected:**
- Clear error message
- Error details displayed
- Suggestion if applicable
- Exit with error code

**Actual:** [ ]

**Issues Found:**

---

### 3.4 Config Utility Commands

#### Test Case 16: Config Validate

**Steps:**
1. Create valid config
2. Run: `dbarena config validate`
3. Create invalid config (syntax error)
4. Run: `dbarena config validate`

**Expected:**
- Valid config: "✓ Configuration valid"
- Invalid config: Clear syntax error with line number

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 17: Config Validate with Script Check

**Steps:**
1. Create config with init scripts
2. Run: `dbarena config validate --check-scripts`

**Expected:**
- Validates config
- Checks all script paths exist
- Reports missing scripts

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 18: Config Show

**Steps:**
1. Create multi-source config (project + user)
2. Run: `dbarena config show`
3. Run: `dbarena config show --profile dev`

**Expected:**
- Shows merged configuration
- Indicates which files loaded
- Profile view shows resolved env vars

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 19: Config Init

**Steps:**
1. In empty directory, run: `dbarena config init`
2. Review generated config

**Expected:**
- Creates `dbarena.toml` with examples
- Well-commented
- Ready to customize

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 20: Init Test Command

**Steps:**
1. Create container with table: `CREATE TABLE test (id INT);`
2. Create script: `INSERT INTO test VALUES (1), (2);`
3. Run: `dbarena init test ./script.sql --container <id>`

**Expected:**
- Script executes against container
- Success message with stats
- Output displayed
- Log saved

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 21: Init Validate Command

**Steps:**
1. Create valid SQL file
2. Run: `dbarena init validate ./script.sql --database postgres`
3. Create file with typos
4. Run validate again

**Expected:**
- Basic validation (syntax patterns, typos)
- Warnings for potential issues
- Note about testing against real DB

**Actual:** [ ]

**Issues Found:**

---

### 3.5 Error Scenarios

#### Test Case 22: Profile Not Found

**Steps:**
1. Run: `dbarena create postgres --profile nonexistent`

**Expected:**
- Clear error: "Profile not found: nonexistent"
- Suggestion of similar profiles
- List of available profiles

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 23: Script File Not Found

**Steps:**
1. Run: `dbarena create postgres --init-script ./missing.sql`

**Expected:**
- Error before container creation
- "Script not found: ./missing.sql"
- Current directory shown
- Suggestions if similar files exist

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 24: Invalid Env Var Format

**Steps:**
1. Run: `dbarena create postgres --env INVALID`

**Expected:**
- Error: "Invalid format, expected KEY=VALUE"
- Example shown

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 25: Config Syntax Error

**Steps:**
1. Create malformed config
2. Run: `dbarena create postgres`

**Expected:**
- Parse error with line number
- Snippet of problematic section
- Suggestion for fix

**Actual:** [ ]

**Issues Found:**

---

### 3.6 Backwards Compatibility

#### Test Case 26: No Config (v0.1.0 Behavior)

**Steps:**
1. Ensure no config files exist
2. Run: `dbarena create postgres`

**Expected:**
- Works exactly as v0.1.0
- Default env vars used
- No warnings or messages about config

**Actual:** [ ]

**Issues Found:**

---

#### Test Case 27: All v0.1.0 Commands

**Steps:**
1. Test each v0.1.0 command without config:
   - `dbarena create postgres`
   - `dbarena list`
   - `dbarena stop <name>`
   - `dbarena start <name>`
   - `dbarena restart <name>`
   - `dbarena inspect <name>`
   - `dbarena logs <name>`
   - `dbarena destroy <name>`

**Expected:**
- All commands work unchanged
- No new prompts or behaviors
- Same output format

**Actual:** [ ]

**Issues Found:**

---

## Phase 4: Performance Testing

### 4.1 Config Loading Performance

**Test:** Measure config file loading time

```bash
# Create large config with 50 profiles
time dbarena config show
```

**Target:** < 10ms

**Actual:** _____ ms

**Pass/Fail:** [ ]

---

### 4.2 Container Creation Overhead

**Test:** Compare v0.1.0 vs v0.2.0 creation time

```bash
# Without config (baseline)
time dbarena create postgres --name baseline

# With config (overhead)
time dbarena create postgres --name withconfig --profile dev
```

**Target:** < 10% overhead

**Baseline:** _____ s
**With Config:** _____ s
**Overhead:** _____ %

**Pass/Fail:** [ ]

---

### 4.3 Init Script Execution

**Test:** Measure script execution time

```bash
# Large script with 1000 INSERTs
time dbarena create postgres --init-script ./large-seed.sql
```

**Target:** Linear scaling with script size

**Results:**

| Script Size | Execution Time |
|-------------|----------------|
| 10 statements | _____ s |
| 100 statements | _____ s |
| 1000 statements | _____ s |

**Pass/Fail:** [ ]

---

## Phase 5: Error Recovery Testing

### 5.1 Container Cleanup on Failure

**Test:** Verify containers are cleaned up on script failure

**Steps:**
1. Create bad init script
2. Run creation (should fail)
3. Check: `docker ps -a | grep dbarena`
4. Verify container removed

**Expected:** No orphaned containers

**Actual:** [ ]

---

### 5.2 Partial Script Execution Recovery

**Test:** Recovery from mid-script failure

**Steps:**
1. Create script with 3 statements (2nd fails)
2. Run with continue-on-error
3. Verify 1st statement executed
4. Verify 3rd statement executed
5. Verify 2nd did not

**Expected:** First and third statements succeed

**Actual:** [ ]

---

### 5.3 Config Reload After Fix

**Test:** Fix config and retry

**Steps:**
1. Create invalid config
2. Try to create (fails)
3. Fix config
4. Retry creation

**Expected:** Second attempt succeeds

**Actual:** [ ]

---

## Error Handling Procedures

### When Tests Fail

1. **Capture Context:**
   - Command executed
   - Error message (full output)
   - Docker container state: `docker ps -a`
   - Logs: `dbarena logs <container>`
   - Config files used
   - Environment: OS, Docker version, dbarena version

2. **Reproduce:**
   - Minimal reproduction steps
   - Can you reproduce with verbose mode? `dbarena -vvv ...`

3. **Debug:**
   - Check logs: `~/.local/share/dbarena/logs/`
   - Check Docker container logs: `docker logs <container-id>`
   - Inspect container: `docker inspect <container-id>`
   - Verify file permissions
   - Check disk space

4. **Fix:**
   - Identify root cause
   - Implement fix
   - Add regression test
   - Re-run all affected tests

5. **Document:**
   - Update test results
   - Note issues in this document
   - Create GitHub issue if bug
   - Update documentation if needed

---

## Test Coverage Goals

| Component | Target Coverage | Status |
|-----------|----------------|--------|
| Config Module | 90% | [ ] |
| Init Module | 85% | [ ] |
| Container Module | 80% | [ ] |
| CLI Commands | 75% | [ ] |
| Error Handling | 90% | [ ] |
| **Overall** | **85%** | [ ] |

---

## Testing Tools and Setup

### Required Tools

1. **Rust Testing:**
   ```bash
   cargo test                    # Run unit tests
   cargo test -- --ignored       # Run integration tests
   cargo tarpaulin              # Coverage (install: cargo install cargo-tarpaulin)
   ```

2. **Docker:**
   - Docker Desktop or Docker Engine running
   - Images: postgres:16, mysql:8.0, mcr.microsoft.com/mssql/server:2022-latest

3. **Manual Testing:**
   - Terminal with color support
   - Database clients (psql, mysql, sqlcmd)
   - Text editor for config files

### Test Environment Setup

```bash
# 1. Build release binary
cargo build --release

# 2. Create test directory
mkdir -p /tmp/dbarena-test
cd /tmp/dbarena-test

# 3. Add dbarena to PATH
export PATH="/path/to/dbarena/target/release:$PATH"

# 4. Verify
dbarena --version

# 5. Clean slate
dbarena list | grep dbarena- | awk '{print $1}' | xargs -I {} dbarena destroy {} -y
```

---

## Test Execution Checklist

### Phase 1: Unit Tests
- [ ] Config parsing tests
- [ ] Profile resolution tests
- [ ] Env var merging tests
- [ ] Error parser tests
- [ ] Log management tests
- [ ] Container config tests

### Phase 2: Integration Tests
- [ ] Config loading integration
- [ ] Init script execution (Postgres)
- [ ] Init script execution (MySQL)
- [ ] Init script execution (SQL Server)
- [ ] Exec command tests
- [ ] Config utility tests

### Phase 3: Manual Tests
- [ ] Configuration management (6 tests)
- [ ] Initialization scripts (6 tests)
- [ ] SQL execution (4 tests)
- [ ] Config utilities (6 tests)
- [ ] Error scenarios (4 tests)
- [ ] Backwards compatibility (2 tests)

### Phase 4: Performance Tests
- [ ] Config loading performance
- [ ] Creation overhead
- [ ] Init script scaling

### Phase 5: Error Recovery
- [ ] Cleanup on failure
- [ ] Partial execution recovery
- [ ] Config reload after fix

---

## Success Criteria

v0.2.0 testing is complete when:

1. ✅ All unit tests pass (100%)
2. ✅ All integration tests pass (100%)
3. ✅ 90%+ manual test cases pass
4. ✅ Performance targets met
5. ✅ Test coverage >85%
6. ✅ All critical bugs fixed
7. ✅ Documentation updated with test results
8. ✅ No regressions from v0.1.0

---

## Notes and Observations

### Bugs Found

| ID | Description | Severity | Status | Fix |
|----|-------------|----------|--------|-----|
| | | | | |

### Performance Issues

| Issue | Target | Actual | Status | Fix |
|-------|--------|--------|--------|-----|
| | | | | |

### Documentation Gaps

| Gap | Priority | Status |
|-----|----------|--------|
| | | |

---

## Final Sign-Off

**Date:** __________

**Unit Tests:** [ ] Pass / [ ] Fail

**Integration Tests:** [ ] Pass / [ ] Fail

**Manual Tests:** [ ] Pass / [ ] Fail

**Performance:** [ ] Pass / [ ] Fail

**Coverage:** _____%

**Critical Bugs:** [ ] None / [ ] Fixed / [ ] Known Issues

**Ready for Release:** [ ] Yes / [ ] No

**Notes:**

---

## Next Steps After Testing

1. Fix all critical bugs
2. Address performance issues
3. Update documentation based on findings
4. Create release notes
5. Tag release: `git tag v0.2.0`
6. Build release binaries
7. Update README with test results
8. Announce release

---

*End of Testing Plan*
