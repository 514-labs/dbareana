use bollard::Docker;
use bollard::volume::{CreateVolumeOptions, ListVolumesOptions, RemoveVolumeOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{DBArenaError, Result};

/// Volume configuration for creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeConfig {
    /// Volume name
    pub name: String,
    /// Container mount path
    pub mount_path: String,
    /// Volume driver (usually "local")
    pub driver: String,
    /// Driver-specific options
    pub driver_opts: HashMap<String, String>,
}

impl VolumeConfig {
    pub fn new(name: String, mount_path: String) -> Self {
        Self {
            name,
            mount_path,
            driver: "local".to_string(),
            driver_opts: HashMap::new(),
        }
    }
}

/// Volume mount specification for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Volume name (for named volumes) or host path (for bind mounts)
    pub source: String,
    /// Container path where volume is mounted
    pub target: String,
    /// Read-only mount
    pub read_only: bool,
    /// Mount type: "volume" or "bind"
    pub mount_type: VolumeMountType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VolumeMountType {
    Volume,
    Bind,
}

impl VolumeMount {
    /// Create a named volume mount
    pub fn volume(name: String, container_path: String, read_only: bool) -> Self {
        Self {
            source: name,
            target: container_path,
            read_only,
            mount_type: VolumeMountType::Volume,
        }
    }

    /// Create a bind mount
    pub fn bind(host_path: String, container_path: String, read_only: bool) -> Self {
        Self {
            source: host_path,
            target: container_path,
            read_only,
            mount_type: VolumeMountType::Bind,
        }
    }

    /// Convert to Docker API mount format
    pub fn to_docker_mount(&self) -> bollard::models::Mount {
        bollard::models::Mount {
            target: Some(self.target.clone()),
            source: Some(self.source.clone()),
            typ: Some(match self.mount_type {
                VolumeMountType::Volume => bollard::models::MountTypeEnum::VOLUME,
                VolumeMountType::Bind => bollard::models::MountTypeEnum::BIND,
            }),
            read_only: Some(self.read_only),
            ..Default::default()
        }
    }
}

/// Volume manager for Docker volume operations
pub struct VolumeManager {
    docker: Arc<Docker>,
}

impl VolumeManager {
    pub fn new(docker: Arc<Docker>) -> Self {
        Self { docker }
    }

    /// Create a new Docker volume
    pub async fn create(&self, config: VolumeConfig) -> Result<String> {
        let mut options = CreateVolumeOptions {
            name: config.name.clone(),
            driver: config.driver.clone(),
            ..Default::default()
        };

        if !config.driver_opts.is_empty() {
            options.driver_opts = config.driver_opts.clone();
        }

        // Add dbarena label
        let mut labels = HashMap::new();
        labels.insert("dbarena.managed".to_string(), "true".to_string());
        options.labels = labels;

        let volume = self
            .docker
            .create_volume(options)
            .await
            .map_err(|e| DBArenaError::VolumeError(format!("Failed to create volume: {}", e)))?;

        Ok(volume.name)
    }

    /// List Docker volumes
    pub async fn list(&self, managed_only: bool) -> Result<Vec<VolumeInfo>> {
        let mut filters = HashMap::new();
        if managed_only {
            filters.insert("label".to_string(), vec!["dbarena.managed=true".to_string()]);
        }

        let options = ListVolumesOptions {
            filters,
        };

        let response = self
            .docker
            .list_volumes(Some(options))
            .await
            .map_err(|e| DBArenaError::VolumeError(format!("Failed to list volumes: {}", e)))?;

        let volumes = response
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(|v| VolumeInfo {
                name: v.name,
                driver: v.driver,
                mountpoint: v.mountpoint,
                created_at: v.created_at,
            })
            .collect();

        Ok(volumes)
    }

    /// Delete a volume
    pub async fn delete(&self, name: &str, force: bool) -> Result<()> {
        let options = Some(RemoveVolumeOptions { force });

        self.docker
            .remove_volume(name, options)
            .await
            .map_err(|e| DBArenaError::VolumeError(format!("Failed to delete volume: {}", e)))?;

        Ok(())
    }

    /// Inspect a volume
    pub async fn inspect(&self, name: &str) -> Result<VolumeDetails> {
        let volume = self
            .docker
            .inspect_volume(name)
            .await
            .map_err(|e| DBArenaError::VolumeError(format!("Failed to inspect volume: {}", e)))?;

        Ok(VolumeDetails {
            name: volume.name,
            driver: volume.driver,
            mountpoint: volume.mountpoint,
            created_at: volume.created_at,
            labels: volume.labels,
            options: volume.options,
        })
    }
}

/// Volume information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created_at: Option<String>,
}

/// Detailed volume information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeDetails {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created_at: Option<String>,
    pub labels: HashMap<String, String>,
    pub options: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_mount_creation() {
        let mount = VolumeMount::volume(
            "test-volume".to_string(),
            "/data".to_string(),
            false,
        );

        assert_eq!(mount.source, "test-volume");
        assert_eq!(mount.target, "/data");
        assert_eq!(mount.read_only, false);
        assert_eq!(mount.mount_type, VolumeMountType::Volume);
    }

    #[test]
    fn test_bind_mount_creation() {
        let mount = VolumeMount::bind(
            "/host/path".to_string(),
            "/container/path".to_string(),
            true,
        );

        assert_eq!(mount.source, "/host/path");
        assert_eq!(mount.target, "/container/path");
        assert_eq!(mount.read_only, true);
        assert_eq!(mount.mount_type, VolumeMountType::Bind);
    }

    #[test]
    fn test_volume_config_creation() {
        let config = VolumeConfig::new(
            "my-volume".to_string(),
            "/data".to_string(),
        );

        assert_eq!(config.name, "my-volume");
        assert_eq!(config.mount_path, "/data");
        assert_eq!(config.driver, "local");
        assert!(config.driver_opts.is_empty());
    }
}
