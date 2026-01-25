use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level configuration structure
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DBArenaConfig {
    /// Configuration version
    pub version: Option<String>,

    /// Global default settings applied to all containers
    #[serde(default)]
    pub defaults: DefaultsConfig,

    /// Global environment profiles
    #[serde(default)]
    pub profiles: HashMap<String, ProfileConfig>,

    /// Database-specific configurations
    #[serde(default)]
    pub databases: HashMap<String, DatabaseConfig>,

    /// Performance monitoring configuration
    #[serde(default)]
    pub monitoring: MonitoringConfig,

    /// Snapshot configuration
    #[serde(default)]
    pub snapshots: SnapshotsConfig,
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

    /// Auto-create volume for data persistence
    pub auto_volume: Option<bool>,

    /// Volume path inside container (database-specific default)
    pub volume_path: Option<String>,

    /// Named volumes to mount
    #[serde(default)]
    pub volumes: Vec<VolumeSpec>,

    /// Bind mounts to create
    #[serde(default)]
    pub bind_mounts: Vec<BindMountSpec>,
}

/// Volume specification
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VolumeSpec {
    /// Volume name
    pub name: String,
    /// Container path
    pub path: String,
    /// Read-only mount
    #[serde(default)]
    pub read_only: bool,
}

/// Bind mount specification
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BindMountSpec {
    /// Host path
    pub host: String,
    /// Container path
    pub container: String,
    /// Read-only mount
    #[serde(default)]
    pub read_only: bool,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    /// Enable monitoring features
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Metrics collection interval in seconds
    #[serde(default = "default_interval")]
    pub interval_seconds: u64,

    /// CPU usage warning threshold (percentage)
    #[serde(default = "default_cpu_warning")]
    pub cpu_warning_threshold: f64,

    /// Memory usage warning threshold (percentage)
    #[serde(default = "default_memory_warning")]
    pub memory_warning_threshold: f64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 2,
            cpu_warning_threshold: 75.0,
            memory_warning_threshold: 80.0,
        }
    }
}

/// Snapshot configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SnapshotsConfig {
    /// Auto-pause container during snapshot creation
    #[serde(default = "default_true")]
    pub auto_pause: bool,

    /// Storage path for snapshot metadata
    pub storage_path: Option<String>,

    /// Maximum number of snapshots per container
    #[serde(default = "default_max_snapshots")]
    pub max_snapshots_per_container: usize,
}

impl Default for SnapshotsConfig {
    fn default() -> Self {
        Self {
            auto_pause: true,
            storage_path: None,
            max_snapshots_per_container: 10,
        }
    }
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_interval() -> u64 {
    1  // 1 second for responsive real-time monitoring
}

fn default_cpu_warning() -> f64 {
    75.0
}

fn default_memory_warning() -> f64 {
    80.0
}

fn default_max_snapshots() -> usize {
    10
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
