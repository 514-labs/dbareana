/// Unit tests for v0.1.0 features
/// Tests core functionality: container config builder, database types, connection strings, etc.
use dbarena::container::{ContainerConfig, DatabaseType};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_database_type_from_string() {
    // PostgreSQL variants
    assert_eq!(
        DatabaseType::from_string("postgres"),
        Some(DatabaseType::Postgres)
    );
    assert_eq!(
        DatabaseType::from_string("postgresql"),
        Some(DatabaseType::Postgres)
    );
    assert_eq!(
        DatabaseType::from_string("pg"),
        Some(DatabaseType::Postgres)
    );
    assert_eq!(
        DatabaseType::from_string("POSTGRES"),
        Some(DatabaseType::Postgres)
    );

    // MySQL variants
    assert_eq!(
        DatabaseType::from_string("mysql"),
        Some(DatabaseType::MySQL)
    );
    assert_eq!(
        DatabaseType::from_string("mariadb"),
        Some(DatabaseType::MySQL)
    );
    assert_eq!(
        DatabaseType::from_string("MYSQL"),
        Some(DatabaseType::MySQL)
    );

    // SQL Server variants
    assert_eq!(
        DatabaseType::from_string("sqlserver"),
        Some(DatabaseType::SQLServer)
    );
    assert_eq!(
        DatabaseType::from_string("mssql"),
        Some(DatabaseType::SQLServer)
    );
    assert_eq!(
        DatabaseType::from_string("sql-server"),
        Some(DatabaseType::SQLServer)
    );
    assert_eq!(
        DatabaseType::from_string("SQLSERVER"),
        Some(DatabaseType::SQLServer)
    );

    // Invalid
    assert_eq!(DatabaseType::from_string("oracle"), None);
    assert_eq!(DatabaseType::from_string(""), None);
}

#[test]
fn test_database_type_default_versions() {
    assert_eq!(DatabaseType::Postgres.default_version(), "16");
    assert_eq!(DatabaseType::MySQL.default_version(), "8.0");
    assert_eq!(DatabaseType::SQLServer.default_version(), "2022-latest");
}

#[test]
fn test_database_type_docker_images() {
    assert_eq!(
        DatabaseType::Postgres.docker_image("15"),
        "postgres:15"
    );
    assert_eq!(
        DatabaseType::MySQL.docker_image("8.0"),
        "mysql:8.0"
    );
    assert_eq!(
        DatabaseType::SQLServer.docker_image("2022-latest"),
        "mcr.microsoft.com/mssql/server:2022-latest"
    );
}

#[test]
fn test_database_type_default_ports() {
    assert_eq!(DatabaseType::Postgres.default_port(), 5432);
    assert_eq!(DatabaseType::MySQL.default_port(), 3306);
    assert_eq!(DatabaseType::SQLServer.default_port(), 1433);
}

#[test]
fn test_database_type_as_str() {
    assert_eq!(DatabaseType::Postgres.as_str(), "postgres");
    assert_eq!(DatabaseType::MySQL.as_str(), "mysql");
    assert_eq!(DatabaseType::SQLServer.as_str(), "sqlserver");
}

#[test]
fn test_container_config_builder_basic() {
    let config = ContainerConfig::new(DatabaseType::Postgres);

    assert_eq!(config.database, DatabaseType::Postgres);
    assert_eq!(config.version, "16");
    assert_eq!(config.name, None);
    assert_eq!(config.port, None);
    assert_eq!(config.persistent, false);
    assert_eq!(config.memory_limit, None);
    assert_eq!(config.cpu_shares, None);
    assert!(config.env_vars.is_empty());
    assert!(config.init_scripts.is_empty());
    assert_eq!(config.continue_on_error, false);
}

#[test]
fn test_container_config_builder_with_all_options() {
    let mut env_vars = HashMap::new();
    env_vars.insert("POSTGRES_DB".to_string(), "testdb".to_string());
    env_vars.insert("POSTGRES_USER".to_string(), "testuser".to_string());

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_version("15".to_string())
        .with_name("test-db".to_string())
        .with_port(5433)
        .with_persistent(true)
        .with_memory_limit(512)
        .with_cpu_shares(1024)
        .with_env_vars(env_vars.clone())
        .with_continue_on_error(true);

    assert_eq!(config.database, DatabaseType::Postgres);
    assert_eq!(config.version, "15");
    assert_eq!(config.name, Some("test-db".to_string()));
    assert_eq!(config.port, Some(5433));
    assert_eq!(config.persistent, true);
    assert_eq!(config.memory_limit, Some(512 * 1024 * 1024)); // Converted to bytes
    assert_eq!(config.cpu_shares, Some(1024));
    assert_eq!(config.env_vars, env_vars);
    assert_eq!(config.continue_on_error, true);
}

#[test]
fn test_container_config_memory_conversion() {
    // Memory limit should be converted from MB to bytes
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_memory_limit(256);

    assert_eq!(config.memory_limit, Some(256 * 1024 * 1024));

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_memory_limit(1024);

    assert_eq!(config.memory_limit, Some(1024 * 1024 * 1024));
}

#[test]
fn test_container_config_env_var_single() {
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_env_var("POSTGRES_DB".to_string(), "testdb".to_string())
        .with_env_var("POSTGRES_USER".to_string(), "testuser".to_string());

    assert_eq!(config.env_vars.len(), 2);
    assert_eq!(
        config.env_vars.get("POSTGRES_DB"),
        Some(&"testdb".to_string())
    );
    assert_eq!(
        config.env_vars.get("POSTGRES_USER"),
        Some(&"testuser".to_string())
    );
}

#[test]
fn test_container_config_init_scripts() {
    let script1 = PathBuf::from("/path/to/script1.sql");
    let script2 = PathBuf::from("/path/to/script2.sql");

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_init_script(script1.clone())
        .with_init_script(script2.clone());

    assert_eq!(config.init_scripts.len(), 2);
    assert_eq!(config.init_scripts[0], script1);
    assert_eq!(config.init_scripts[1], script2);
}

#[test]
fn test_container_config_init_scripts_batch() {
    let scripts = vec![
        PathBuf::from("/path/to/script1.sql"),
        PathBuf::from("/path/to/script2.sql"),
        PathBuf::from("/path/to/script3.sql"),
    ];

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_init_scripts(scripts.clone());

    assert_eq!(config.init_scripts.len(), 3);
    assert_eq!(config.init_scripts, scripts);
}

#[test]
fn test_container_config_mysql() {
    let config = ContainerConfig::new(DatabaseType::MySQL)
        .with_version("8.0".to_string())
        .with_name("test-mysql".to_string());

    assert_eq!(config.database, DatabaseType::MySQL);
    assert_eq!(config.version, "8.0");
    assert_eq!(config.name, Some("test-mysql".to_string()));
}

#[test]
fn test_container_config_sqlserver() {
    let config = ContainerConfig::new(DatabaseType::SQLServer)
        .with_version("2022-latest".to_string())
        .with_name("test-sqlserver".to_string());

    assert_eq!(config.database, DatabaseType::SQLServer);
    assert_eq!(config.version, "2022-latest");
    assert_eq!(config.name, Some("test-sqlserver".to_string()));
}

#[test]
fn test_container_config_builder_chaining() {
    // Test that builder pattern allows for flexible chaining
    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_version("14".to_string())
        .with_name("test".to_string())
        .with_port(5555)
        .with_persistent(true);

    assert_eq!(config.version, "14");
    assert_eq!(config.name, Some("test".to_string()));
    assert_eq!(config.port, Some(5555));
    assert_eq!(config.persistent, true);
}

#[test]
fn test_database_type_display() {
    assert_eq!(format!("{}", DatabaseType::Postgres), "postgres");
    assert_eq!(format!("{}", DatabaseType::MySQL), "mysql");
    assert_eq!(format!("{}", DatabaseType::SQLServer), "sqlserver");
}
