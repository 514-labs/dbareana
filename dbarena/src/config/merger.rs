use super::schema::{DBArenaConfig, DatabaseConfig, DefaultsConfig, ProfileConfig};
use std::collections::HashMap;

/// Merge two configurations with override taking precedence
///
/// Merging rules:
/// - Defaults: override values replace base values
/// - Profiles: override profiles replace base profiles (no deep merge)
/// - Databases: merge database configs, with override taking precedence
/// - Environment variables: override replaces base
pub fn merge_configs(base: DBArenaConfig, override_config: DBArenaConfig) -> DBArenaConfig {
    DBArenaConfig {
        version: override_config.version.or(base.version),
        defaults: merge_defaults(base.defaults, override_config.defaults),
        profiles: merge_profiles(base.profiles, override_config.profiles),
        databases: merge_databases(base.databases, override_config.databases),
        monitoring: override_config.monitoring, // Override completely replaces
        snapshots: override_config.snapshots,   // Override completely replaces
    }
}

/// Merge default configurations
fn merge_defaults(base: DefaultsConfig, override_config: DefaultsConfig) -> DefaultsConfig {
    DefaultsConfig {
        persistent: override_config.persistent.or(base.persistent),
        memory_mb: override_config.memory_mb.or(base.memory_mb),
        cpu_shares: override_config.cpu_shares.or(base.cpu_shares),
    }
}

/// Merge profile configurations
fn merge_profiles(
    mut base: HashMap<String, ProfileConfig>,
    override_config: HashMap<String, ProfileConfig>,
) -> HashMap<String, ProfileConfig> {
    // Override profiles completely replace base profiles (no env var merging)
    for (name, profile) in override_config {
        base.insert(name, profile);
    }
    base
}

/// Merge database configurations
fn merge_databases(
    mut base: HashMap<String, DatabaseConfig>,
    override_config: HashMap<String, DatabaseConfig>,
) -> HashMap<String, DatabaseConfig> {
    for (db_name, override_db) in override_config {
        let merged = if let Some(base_db) = base.remove(&db_name) {
            merge_database_config(base_db, override_db)
        } else {
            override_db
        };
        base.insert(db_name, merged);
    }
    base
}

/// Merge two database configurations
fn merge_database_config(base: DatabaseConfig, override_config: DatabaseConfig) -> DatabaseConfig {
    let mut env = base.env;
    env.extend(override_config.env);

    let mut profiles = base.profiles;
    profiles.extend(override_config.profiles);

    let mut init_scripts = base.init_scripts;
    init_scripts.extend(override_config.init_scripts);

    let mut volumes = base.volumes;
    volumes.extend(override_config.volumes);

    let mut bind_mounts = base.bind_mounts;
    bind_mounts.extend(override_config.bind_mounts);

    DatabaseConfig {
        default_version: override_config.default_version.or(base.default_version),
        env,
        profiles,
        init_scripts,
        auto_volume: override_config.auto_volume.or(base.auto_volume),
        volume_path: override_config.volume_path.or(base.volume_path),
        volumes,
        bind_mounts,
    }
}

/// Apply CLI environment variable overrides to a config
///
/// CLI env vars have the highest precedence and override all config values
pub fn apply_cli_overrides(_config: &mut DBArenaConfig, _cli_env: HashMap<String, String>) {
    // CLI overrides are not part of the config structure itself
    // They will be applied at the container creation stage
    // This function exists for future use if we want to pre-apply CLI overrides to the config
}

/// Merge environment variables with precedence
///
/// Later entries override earlier ones
pub fn merge_env_vars(layers: Vec<HashMap<String, String>>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for layer in layers {
        result.extend(layer);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_defaults() {
        let base = DefaultsConfig {
            persistent: Some(false),
            memory_mb: Some(512),
            cpu_shares: None,
        };

        let override_config = DefaultsConfig {
            persistent: None,
            memory_mb: Some(1024),
            cpu_shares: Some(2048),
        };

        let merged = merge_defaults(base, override_config);
        assert_eq!(merged.persistent, Some(false)); // Base preserved
        assert_eq!(merged.memory_mb, Some(1024)); // Override wins
        assert_eq!(merged.cpu_shares, Some(2048)); // Override adds
    }

    #[test]
    fn test_merge_profiles() {
        let mut base = HashMap::new();
        base.insert(
            "dev".to_string(),
            ProfileConfig {
                env: [("LOG_LEVEL".to_string(), "debug".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
            },
        );

        let mut override_config = HashMap::new();
        override_config.insert(
            "dev".to_string(),
            ProfileConfig {
                env: [("LOG_LEVEL".to_string(), "info".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
            },
        );
        override_config.insert(
            "prod".to_string(),
            ProfileConfig {
                env: [("LOG_LEVEL".to_string(), "error".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
            },
        );

        let merged = merge_profiles(base, override_config);
        assert_eq!(merged.len(), 2);
        assert_eq!(
            merged.get("dev").unwrap().env.get("LOG_LEVEL"),
            Some(&"info".to_string())
        );
        assert_eq!(
            merged.get("prod").unwrap().env.get("LOG_LEVEL"),
            Some(&"error".to_string())
        );
    }

    #[test]
    fn test_merge_database_config() {
        let base = DatabaseConfig {
            default_version: Some("14".to_string()),
            env: [("POSTGRES_DB".to_string(), "basedb".to_string())]
                .iter()
                .cloned()
                .collect(),
            profiles: HashMap::new(),
            init_scripts: vec![],
            auto_volume: None,
            volume_path: None,
            volumes: vec![],
            bind_mounts: vec![],
        };

        let override_config = DatabaseConfig {
            default_version: Some("16".to_string()),
            env: [
                ("POSTGRES_DB".to_string(), "overridedb".to_string()),
                ("POSTGRES_USER".to_string(), "user".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            profiles: HashMap::new(),
            init_scripts: vec![],
            auto_volume: None,
            volume_path: None,
            volumes: vec![],
            bind_mounts: vec![],
        };

        let merged = merge_database_config(base, override_config);
        assert_eq!(merged.default_version, Some("16".to_string()));
        assert_eq!(
            merged.env.get("POSTGRES_DB"),
            Some(&"overridedb".to_string())
        );
        assert_eq!(merged.env.get("POSTGRES_USER"), Some(&"user".to_string()));
    }

    #[test]
    fn test_merge_env_vars() {
        let layer1 = [
            ("A".to_string(), "1".to_string()),
            ("B".to_string(), "2".to_string()),
        ]
        .iter()
        .cloned()
        .collect();

        let layer2 = [
            ("B".to_string(), "3".to_string()),
            ("C".to_string(), "4".to_string()),
        ]
        .iter()
        .cloned()
        .collect();

        let merged = merge_env_vars(vec![layer1, layer2]);
        assert_eq!(merged.get("A"), Some(&"1".to_string()));
        assert_eq!(merged.get("B"), Some(&"3".to_string())); // Layer2 wins
        assert_eq!(merged.get("C"), Some(&"4".to_string()));
    }

    #[test]
    fn test_merge_complete_configs() {
        let base_toml = r#"
            [defaults]
            persistent = false
            memory_mb = 512

            [profiles.dev]
            env = { LOG_LEVEL = "debug" }

            [databases.postgres.env]
            POSTGRES_DB = "basedb"
        "#;

        let override_toml = r#"
            [defaults]
            memory_mb = 1024

            [profiles.prod]
            env = { LOG_LEVEL = "error" }

            [databases.postgres.env]
            POSTGRES_DB = "overridedb"
            POSTGRES_USER = "user"
        "#;

        let base: DBArenaConfig = toml::from_str(base_toml).unwrap();
        let override_config: DBArenaConfig = toml::from_str(override_toml).unwrap();

        let merged = merge_configs(base, override_config);

        assert_eq!(merged.defaults.persistent, Some(false));
        assert_eq!(merged.defaults.memory_mb, Some(1024));
        assert_eq!(merged.profiles.len(), 2); // dev and prod
        assert_eq!(
            merged
                .databases
                .get("postgres")
                .unwrap()
                .env
                .get("POSTGRES_DB"),
            Some(&"overridedb".to_string())
        );
    }
}
