use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub database: DatabaseType,
    pub version: String,
    pub name: Option<String>,
    pub port: Option<u16>,
    pub persistent: bool,
    pub memory_limit: Option<u64>,
    pub cpu_shares: Option<u64>,
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
}
