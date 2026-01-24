use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level configuration structure
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DBArenaConfig {
    /// Global default settings applied to all containers
    #[serde(default)]
    pub defaults: DefaultsConfig,

    /// Global environment profiles
    #[serde(default)]
    pub profiles: HashMap<String, ProfileConfig>,

    /// Database-specific configurations
    #[serde(default)]
    pub databases: HashMap<String, DatabaseConfig>,
}

/// Default settings for all containers
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DefaultsConfig {
    /// Whether containers are persistent by default
    pub persistent: Option<bool>,

    /// Default memory limit in MB
    pub memory_mb: Option<u64>,

    /// Default CPU shares
    pub cpu_shares: Option<u64>,
}

/// Environment profile configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProfileConfig {
    /// Environment variables for this profile
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Database-specific configuration
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DatabaseConfig {
    /// Default version to use for this database type
    pub default_version: Option<String>,

    /// Base environment variables for this database
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Database-specific profiles (override global profiles)
    #[serde(default)]
    pub profiles: HashMap<String, ProfileConfig>,

    /// Initialization scripts to run on container startup
    #[serde(default)]
    pub init_scripts: Vec<InitScript>,
}

/// Initialization script configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum InitScript {
    /// Simple script path as a string
    Simple(String),

    /// Detailed configuration with options
    Detailed {
        /// Path to the script file (supports glob patterns)
        path: String,

        /// Continue even if this script fails
        #[serde(default)]
        continue_on_error: bool,
    },
}

impl InitScript {
    /// Get the path from the script configuration
    pub fn path(&self) -> &str {
        match self {
            InitScript::Simple(path) => path,
            InitScript::Detailed { path, .. } => path,
        }
    }

    /// Check if the script should continue on error
    pub fn continue_on_error(&self) -> bool {
        match self {
            InitScript::Simple(_) => false,
            InitScript::Detailed { continue_on_error, .. } => *continue_on_error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
            [defaults]
            persistent = false
        "#;

        let config: DBArenaConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.defaults.persistent, Some(false));
    }

    #[test]
    fn test_parse_complete_config() {
        let toml = r#"
            [defaults]
            persistent = false
            memory_mb = 512

            [profiles.dev]
            env = { LOG_LEVEL = "debug" }

            [databases.postgres.env]
            POSTGRES_USER = "appuser"
            POSTGRES_PASSWORD = "dev123"

            [databases.postgres.profiles.dev]
            env = { POSTGRES_DB = "myapp_dev" }
        "#;

        let config: DBArenaConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.defaults.memory_mb, Some(512));
        assert!(config.profiles.contains_key("dev"));
        assert!(config.databases.contains_key("postgres"));
    }

    #[test]
    fn test_init_script_simple() {
        let toml = r#"
            [databases.postgres]
            init_scripts = ["./schema.sql"]
        "#;

        let config: DBArenaConfig = toml::from_str(toml).unwrap();
        let scripts = &config.databases.get("postgres").unwrap().init_scripts;
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].path(), "./schema.sql");
        assert!(!scripts[0].continue_on_error());
    }

    #[test]
    fn test_init_script_detailed() {
        let toml = r#"
            [[databases.postgres.init_scripts]]
            path = "./schema.sql"
            continue_on_error = true
        "#;

        let config: DBArenaConfig = toml::from_str(toml).unwrap();
        let scripts = &config.databases.get("postgres").unwrap().init_scripts;
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].path(), "./schema.sql");
        assert!(scripts[0].continue_on_error());
    }
}
