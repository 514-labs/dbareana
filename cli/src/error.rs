use thiserror::Error;

#[derive(Debug, Error)]
pub enum DBArenaError {
    #[error("Docker daemon not running or not accessible")]
    DockerNotAvailable,

    #[error("Docker operation failed: {0}")]
    DockerError(#[from] bollard::errors::Error),

    #[error("Container not found: {0}")]
    ContainerNotFound(String),

    #[error("Container health check timed out after {0} seconds")]
    HealthCheckTimeout(u64),

    #[error("Port {0} is already in use")]
    PortInUse(u16),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Image pull failed: {0}")]
    ImagePullFailed(String),

    #[error("Container operation failed: {0}")]
    ContainerOperationFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Invalid environment variable: {0}")]
    InvalidEnvVar(String),

    #[error("Initialization script failed: {0}")]
    InitScriptFailed(String),

    #[error("Initialization script not found: {0}")]
    InitScriptNotFound(String),

    #[error("Monitoring error: {0}")]
    MonitoringError(String),

    #[error("Snapshot error: {0}")]
    SnapshotError(String),

    #[error("Volume error: {0}")]
    VolumeError(String),

    #[error("Docs error: {0}")]
    DocsError(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DBArenaError>;
