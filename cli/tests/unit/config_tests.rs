/// Unit tests for v0.2.0 config features
/// Tests config parsing, profile resolution, env var precedence, validation
use dbarena::config::loader::{load_config_from_string, ConfigFormat};
use dbarena::config::merger::{merge_configs, merge_env_vars};
use dbarena::config::profile::{resolve_profile, list_profiles, get_database_env};
use dbarena::config::schema::{DBArenaConfig, InitScript};
use dbarena::container::DatabaseType;
use std::collections::HashMap;

#[test]
fn test_parse_toml_basic_config() {
    let toml = r#"
        [databases.postgres]
        version = "16"

        [databases.postgres.env]
        POSTGRES_DB = "testdb"
        POSTGRES_USER = "testuser"
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    assert!(config.databases.contains_key("postgres"));
    let pg_config = &config.databases["postgres"];
    assert_eq!(pg_config.env.get("POSTGRES_DB"), Some(&"testdb".to_string()));
    assert_eq!(
        pg_config.env.get("POSTGRES_USER"),
        Some(&"testuser".to_string())
    );
}

#[test]
fn test_parse_yaml_basic_config() {
    let yaml = r#"
databases:
  postgres:
    version: "16"
    env:
      POSTGRES_DB: "testdb"
      POSTGRES_USER: "testuser"
    "#;

    let config: DBArenaConfig =
        load_config_from_string(yaml, ConfigFormat::Yaml).expect("Failed to parse YAML");

    assert!(config.databases.contains_key("postgres"));
    let pg_config = &config.databases["postgres"];
    assert_eq!(pg_config.env.get("POSTGRES_DB"), Some(&"testdb".to_string()));
}

#[test]
fn test_parse_config_with_profiles() {
    let toml = r#"
        [profiles.dev]
        [profiles.dev.env]
        LOG_LEVEL = "debug"
        ENV_NAME = "development"

        [profiles.prod]
        [profiles.prod.env]
        LOG_LEVEL = "error"
        ENV_NAME = "production"
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    assert_eq!(config.profiles.len(), 2);
    assert!(config.profiles.contains_key("dev"));
    assert!(config.profiles.contains_key("prod"));

    let dev_env = &config.profiles["dev"].env;
    assert_eq!(dev_env.get("LOG_LEVEL"), Some(&"debug".to_string()));
}

#[test]
fn test_parse_config_with_init_scripts_simple() {
    let toml = r#"
        [databases.postgres]
        init_scripts = ["schema.sql", "seed.sql"]
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    let scripts = &config.databases["postgres"].init_scripts;
    assert_eq!(scripts.len(), 2);

    assert_eq!(scripts[0].path(), "schema.sql");
    assert!(!scripts[0].continue_on_error());

    assert_eq!(scripts[1].path(), "seed.sql");
    assert!(!scripts[1].continue_on_error());
}

#[test]
fn test_parse_config_with_init_scripts_detailed() {
    let toml = r#"
        [[databases.postgres.init_scripts]]
        path = "schema.sql"
        continue_on_error = false

        [[databases.postgres.init_scripts]]
        path = "seed.sql"
        continue_on_error = true
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    let scripts = &config.databases["postgres"].init_scripts;
    assert_eq!(scripts.len(), 2);

    assert_eq!(scripts[0].path(), "schema.sql");
    assert!(!scripts[0].continue_on_error());

    assert_eq!(scripts[1].path(), "seed.sql");
    assert!(scripts[1].continue_on_error());
}

#[test]
fn test_parse_config_with_defaults() {
    let toml = r#"
        [defaults]
        persistent = true
        memory_mb = 512
        cpu_shares = 1024
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    assert_eq!(config.defaults.persistent, Some(true));
    assert_eq!(config.defaults.memory_mb, Some(512));
    assert_eq!(config.defaults.cpu_shares, Some(1024));
}

#[test]
fn test_profile_resolution_global() {
    let toml = r#"
        [profiles.dev]
        [profiles.dev.env]
        LOG_LEVEL = "debug"
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    let env = resolve_profile(&config, "dev", DatabaseType::Postgres).expect("Profile not found");
    assert_eq!(env.get("LOG_LEVEL"), Some(&"debug".to_string()));
}

#[test]
fn test_profile_resolution_database_specific() {
    let toml = r#"
        [profiles.dev]
        [profiles.dev.env]
        LOG_LEVEL = "debug"

        [databases.postgres.profiles.dev]
        [databases.postgres.profiles.dev.env]
        POSTGRES_DB = "devdb"
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    let env = resolve_profile(&config, "dev", DatabaseType::Postgres).expect("Profile not found");
    assert_eq!(env.get("LOG_LEVEL"), Some(&"debug".to_string()));
    assert_eq!(env.get("POSTGRES_DB"), Some(&"devdb".to_string()));
}

#[test]
fn test_profile_resolution_database_overrides_global() {
    let toml = r#"
        [profiles.dev]
        [profiles.dev.env]
        LOG_LEVEL = "debug"
        DB_NAME = "global_db"

        [databases.postgres.profiles.dev]
        [databases.postgres.profiles.dev.env]
        DB_NAME = "postgres_db"
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    let env = resolve_profile(&config, "dev", DatabaseType::Postgres).expect("Profile not found");
    assert_eq!(env.get("LOG_LEVEL"), Some(&"debug".to_string()));
    assert_eq!(env.get("DB_NAME"), Some(&"postgres_db".to_string())); // Database-specific wins
}

#[test]
fn test_profile_not_found_error() {
    let config = DBArenaConfig::default();
    let result = resolve_profile(&config, "nonexistent", DatabaseType::Postgres);
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Profile 'nonexistent' not found"));
}

#[test]
fn test_profile_suggestion_levenshtein() {
    let toml = r#"
        [profiles.development]
        [profiles.development.env]
        LOG_LEVEL = "debug"
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    // "developmen" is close enough to "development" (distance 1) to get a suggestion
    let result = resolve_profile(&config, "developmen", DatabaseType::Postgres);
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Did you mean 'development'?"));
}

#[test]
fn test_list_profiles_global_only() {
    let toml = r#"
        [profiles.dev]
        [profiles.dev.env]
        LOG_LEVEL = "debug"

        [profiles.prod]
        [profiles.prod.env]
        LOG_LEVEL = "error"
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    let profiles = list_profiles(&config, DatabaseType::Postgres);
    assert_eq!(profiles.len(), 2);
    assert!(profiles.contains(&"dev".to_string()));
    assert!(profiles.contains(&"prod".to_string()));
}

#[test]
fn test_list_profiles_with_database_specific() {
    let toml = r#"
        [profiles.dev]
        [profiles.dev.env]
        LOG_LEVEL = "debug"

        [databases.postgres.profiles.staging]
        [databases.postgres.profiles.staging.env]
        POSTGRES_DB = "staging"
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    let profiles = list_profiles(&config, DatabaseType::Postgres);
    assert_eq!(profiles.len(), 2);
    assert!(profiles.contains(&"dev".to_string()));
    assert!(profiles.contains(&"staging".to_string()));
}

#[test]
fn test_get_database_env() {
    let toml = r#"
        [databases.postgres]
        [databases.postgres.env]
        POSTGRES_USER = "appuser"
        POSTGRES_PASSWORD = "secret"
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    let env = get_database_env(&config, DatabaseType::Postgres);
    assert_eq!(env.get("POSTGRES_USER"), Some(&"appuser".to_string()));
    assert_eq!(
        env.get("POSTGRES_PASSWORD"),
        Some(&"secret".to_string())
    );
}

#[test]
fn test_merge_env_vars_precedence() {
    let layer1 = HashMap::from([
        ("VAR1".to_string(), "value1".to_string()),
        ("VAR2".to_string(), "value2".to_string()),
    ]);

    let layer2 = HashMap::from([
        ("VAR2".to_string(), "override2".to_string()),
        ("VAR3".to_string(), "value3".to_string()),
    ]);

    let layer3 = HashMap::from([("VAR3".to_string(), "override3".to_string())]);

    let merged = merge_env_vars(vec![layer1, layer2, layer3]);

    assert_eq!(merged.get("VAR1"), Some(&"value1".to_string()));
    assert_eq!(merged.get("VAR2"), Some(&"override2".to_string()));
    assert_eq!(merged.get("VAR3"), Some(&"override3".to_string()));
}

#[test]
fn test_merge_configs_defaults() {
    let base_toml = r#"
        [defaults]
        persistent = false
        memory_mb = 256
    "#;

    let override_toml = r#"
        [defaults]
        memory_mb = 512
        cpu_shares = 1024
    "#;

    let base: DBArenaConfig =
        load_config_from_string(base_toml, ConfigFormat::Toml).expect("Failed to parse base");
    let override_config: DBArenaConfig =
        load_config_from_string(override_toml, ConfigFormat::Toml)
            .expect("Failed to parse override");

    let merged = merge_configs(base, override_config);

    assert_eq!(merged.defaults.persistent, Some(false)); // From base
    assert_eq!(merged.defaults.memory_mb, Some(512)); // Overridden
    assert_eq!(merged.defaults.cpu_shares, Some(1024)); // New
}

#[test]
fn test_merge_configs_profiles() {
    let base_toml = r#"
        [profiles.dev]
        [profiles.dev.env]
        VAR1 = "base1"
    "#;

    let override_toml = r#"
        [profiles.dev]
        [profiles.dev.env]
        VAR2 = "override2"

        [profiles.prod]
        [profiles.prod.env]
        VAR3 = "prod3"
    "#;

    let base: DBArenaConfig =
        load_config_from_string(base_toml, ConfigFormat::Toml).expect("Failed to parse base");
    let override_config: DBArenaConfig =
        load_config_from_string(override_toml, ConfigFormat::Toml)
            .expect("Failed to parse override");

    let merged = merge_configs(base, override_config);

    assert_eq!(merged.profiles.len(), 2);
    // Profiles are completely replaced, not merged
    assert_eq!(
        merged.profiles["dev"].env.get("VAR2"),
        Some(&"override2".to_string())
    );
    assert!(merged.profiles["dev"].env.get("VAR1").is_none()); // Not merged
    assert!(merged.profiles.contains_key("prod"));
}

#[test]
fn test_merge_configs_databases_env() {
    let base_toml = r#"
        [databases.postgres]
        [databases.postgres.env]
        POSTGRES_USER = "baseuser"
        POSTGRES_DB = "basedb"
    "#;

    let override_toml = r#"
        [databases.postgres]
        [databases.postgres.env]
        POSTGRES_DB = "overridedb"
        POSTGRES_PASSWORD = "secret"
    "#;

    let base: DBArenaConfig =
        load_config_from_string(base_toml, ConfigFormat::Toml).expect("Failed to parse base");
    let override_config: DBArenaConfig =
        load_config_from_string(override_toml, ConfigFormat::Toml)
            .expect("Failed to parse override");

    let merged = merge_configs(base, override_config);

    let pg_env = &merged.databases["postgres"].env;
    assert_eq!(pg_env.get("POSTGRES_USER"), Some(&"baseuser".to_string())); // From base
    assert_eq!(
        pg_env.get("POSTGRES_DB"),
        Some(&"overridedb".to_string())
    ); // Overridden
    assert_eq!(
        pg_env.get("POSTGRES_PASSWORD"),
        Some(&"secret".to_string())
    ); // New
}

#[test]
fn test_init_script_enum_simple() {
    let script = InitScript::Simple("schema.sql".to_string());
    assert_eq!(script.path(), "schema.sql");
    assert!(!script.continue_on_error());
}

#[test]
fn test_init_script_enum_detailed() {
    let script = InitScript::Detailed {
        path: "schema.sql".to_string(),
        continue_on_error: true,
    };
    assert_eq!(script.path(), "schema.sql");
    assert!(script.continue_on_error());
}

#[test]
fn test_parse_invalid_toml() {
    let invalid_toml = r#"
        [databases.postgres
        version = "16"
    "#;

    let result = load_config_from_string(invalid_toml, ConfigFormat::Toml);
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_yaml() {
    let invalid_yaml = r#"
databases:
  postgres:
    - invalid
    - structure
    "#;

    let result = load_config_from_string(invalid_yaml, ConfigFormat::Yaml);
    assert!(result.is_err());
}

#[test]
fn test_empty_config() {
    let empty_toml = "";
    let config: DBArenaConfig =
        load_config_from_string(empty_toml, ConfigFormat::Toml).expect("Failed to parse empty");

    assert!(config.profiles.is_empty());
    assert!(config.databases.is_empty());
}

#[test]
fn test_multiple_database_configs() {
    let toml = r#"
        [databases.postgres]
        version = "16"
        [databases.postgres.env]
        POSTGRES_DB = "pgdb"

        [databases.mysql]
        version = "8.0"
        [databases.mysql.env]
        MYSQL_DATABASE = "mydb"
    "#;

    let config: DBArenaConfig =
        load_config_from_string(toml, ConfigFormat::Toml).expect("Failed to parse TOML");

    assert_eq!(config.databases.len(), 2);
    assert!(config.databases.contains_key("postgres"));
    assert!(config.databases.contains_key("mysql"));
}
