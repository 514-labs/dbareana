use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub database: DatabaseType,
    pub version: String,
    pub name: Option<String>,
    pub port: Option<u16>,
    pub persistent: bool,
    pub memory_limit: Option<u64>,
    pub cpu_shares: Option<u64>,
    /// Custom environment variables to set in the container
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    /// Initialization scripts to run after container creation
    #[serde(default)]
    pub init_scripts: Vec<PathBuf>,
    /// Continue creating container even if init scripts fail
    #[serde(default)]
    pub continue_on_error: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Postgres,
    MySQL,
    SQLServer,
}

impl DatabaseType {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "postgres" | "postgresql" | "pg" => Some(DatabaseType::Postgres),
            "mysql" | "mariadb" => Some(DatabaseType::MySQL),
            "sqlserver" | "mssql" | "sql-server" => Some(DatabaseType::SQLServer),
            _ => None,
        }
    }

    pub fn default_version(&self) -> &'static str {
        match self {
            DatabaseType::Postgres => "16",
            DatabaseType::MySQL => "8.0",
            DatabaseType::SQLServer => "2022-latest",
        }
    }

    pub fn docker_image(&self, version: &str) -> String {
        match self {
            DatabaseType::Postgres => format!("postgres:{}", version),
            DatabaseType::MySQL => format!("mysql:{}", version),
            DatabaseType::SQLServer => {
                format!("mcr.microsoft.com/mssql/server:{}", version)
            }
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            DatabaseType::Postgres => 5432,
            DatabaseType::MySQL => 3306,
            DatabaseType::SQLServer => 1433,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseType::Postgres => "postgres",
            DatabaseType::MySQL => "mysql",
            DatabaseType::SQLServer => "sqlserver",
        }
    }
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl ContainerConfig {
    pub fn new(database: DatabaseType) -> Self {
        Self {
            database,
            version: database.default_version().to_string(),
            name: None,
            port: None,
            persistent: false,
            memory_limit: None,
            cpu_shares: None,
            env_vars: HashMap::new(),
            init_scripts: Vec::new(),
            continue_on_error: false,
        }
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn with_persistent(mut self, persistent: bool) -> Self {
        self.persistent = persistent;
        self
    }

    pub fn with_memory_limit(mut self, memory_mb: u64) -> Self {
        self.memory_limit = Some(memory_mb * 1024 * 1024); // Convert MB to bytes
        self
    }

    pub fn with_cpu_shares(mut self, cpu_shares: u64) -> Self {
        self.cpu_shares = Some(cpu_shares);
        self
    }

    pub fn with_env_vars(mut self, env_vars: HashMap<String, String>) -> Self {
        self.env_vars = env_vars;
        self
    }

    pub fn with_env_var(mut self, key: String, value: String) -> Self {
        self.env_vars.insert(key, value);
        self
    }

    pub fn with_init_scripts(mut self, scripts: Vec<PathBuf>) -> Self {
        self.init_scripts = scripts;
        self
    }

    pub fn with_init_script(mut self, script: PathBuf) -> Self {
        self.init_scripts.push(script);
        self
    }

    pub fn with_continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }
}
