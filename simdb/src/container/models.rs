use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub database_type: String,
    pub version: String,
    pub status: ContainerStatus,
    pub port: u16,
    pub host_port: Option<u16>,
    pub persistent: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContainerStatus {
    Creating,
    Starting,
    Running,
    Healthy,
    Unhealthy,
    Stopped,
    Exited,
}

impl std::fmt::Display for ContainerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerStatus::Creating => write!(f, "creating"),
            ContainerStatus::Starting => write!(f, "starting"),
            ContainerStatus::Running => write!(f, "running"),
            ContainerStatus::Healthy => write!(f, "healthy"),
            ContainerStatus::Unhealthy => write!(f, "unhealthy"),
            ContainerStatus::Stopped => write!(f, "stopped"),
            ContainerStatus::Exited => write!(f, "exited"),
        }
    }
}
