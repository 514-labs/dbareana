use bollard::Docker;
use std::sync::Arc;

use super::metadata::Snapshot;
use super::storage::SnapshotStorage;
use crate::container::DatabaseType;
use crate::error::{DBArenaError, Result};

/// Snapshot manager for creating, restoring, and managing snapshots
pub struct SnapshotManager {
    storage: SnapshotStorage,
    docker: Arc<Docker>,
}

impl SnapshotManager {
    pub fn new(docker: Arc<Docker>) -> Self {
        let storage = SnapshotStorage::new(docker.clone());
        Self { storage, docker }
    }

    /// Create a snapshot from a container
    pub async fn create(
        &self,
        container_id: &str,
        name: String,
        message: Option<String>,
        auto_pause: bool,
    ) -> Result<Snapshot> {
        // Get container info to determine database type
        let inspect = self
            .docker
            .inspect_container(container_id, None)
            .await
            .map_err(|_e| DBArenaError::ContainerNotFound(container_id.to_string()))?;

        // Extract database type from labels
        let database_type = inspect
            .config
            .and_then(|c| c.labels)
            .and_then(|labels| labels.get("dbarena.database").cloned())
            .and_then(|db| DatabaseType::from_string(&db))
            .ok_or_else(|| {
                DBArenaError::SnapshotError("Could not determine database type".to_string())
            })?;

        // Create snapshot metadata
        let snapshot = Snapshot::new(name, container_id.to_string(), database_type, message);

        // Commit the container as an image
        self.storage
            .commit_container(container_id, &snapshot, auto_pause)
            .await?;

        tracing::info!(
            "Created snapshot {} from container {}",
            snapshot.id,
            container_id
        );

        Ok(snapshot)
    }

    /// List all snapshots
    pub async fn list(&self) -> Result<Vec<Snapshot>> {
        self.storage.list_snapshots().await
    }

    /// Get a specific snapshot by ID or name
    pub async fn get(&self, id_or_name: &str) -> Result<Snapshot> {
        let snapshots = self.list().await?;

        snapshots
            .into_iter()
            .find(|s| s.id == id_or_name || s.name == id_or_name)
            .ok_or_else(|| {
                DBArenaError::SnapshotError(format!("Snapshot not found: {}", id_or_name))
            })
    }

    /// Restore a snapshot to a new container
    ///
    /// Note: This creates a container from the snapshot image directly,
    /// bypassing normal container creation to preserve the exact snapshot state.
    pub async fn restore(
        &self,
        snapshot_id: &str,
        name: Option<String>,
        port: Option<u16>,
    ) -> Result<crate::container::Container> {
        // Get the snapshot
        let snapshot = self.get(snapshot_id).await?;

        // Create container directly from snapshot image using Docker API
        use bollard::container::{Config, CreateContainerOptions};
        use std::collections::HashMap;

        let container_name =
            name.unwrap_or_else(|| format!("restored-{}-{}", snapshot.name, &snapshot.id[..8]));

        let mut port_bindings = HashMap::new();
        let host_port = port.unwrap_or(0); // 0 means auto-assign

        // Map container port to host port
        let container_port = format!("{}/tcp", snapshot.database_type.default_port());
        port_bindings.insert(
            container_port.clone(),
            Some(vec![bollard::models::PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(host_port.to_string()),
            }]),
        );

        let mut labels = HashMap::new();
        labels.insert("dbarena.managed".to_string(), "true".to_string());
        labels.insert(
            "dbarena.database".to_string(),
            snapshot.database_type.to_string(),
        );
        labels.insert("dbarena.restored_from".to_string(), snapshot.id.clone());

        let config = Config {
            image: Some(snapshot.image_tag.clone()),
            exposed_ports: Some(
                vec![(container_port.clone(), HashMap::new())]
                    .into_iter()
                    .collect(),
            ),
            host_config: Some(bollard::models::HostConfig {
                port_bindings: Some(port_bindings),
                ..Default::default()
            }),
            labels: Some(labels),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: container_name.clone(),
            platform: None,
        };

        let response = self
            .docker
            .create_container(Some(options), config)
            .await
            .map_err(|e| {
                DBArenaError::SnapshotError(format!(
                    "Failed to create container from snapshot: {}",
                    e
                ))
            })?;

        // Start the container
        self.docker
            .start_container::<String>(&response.id, None)
            .await
            .map_err(|e| {
                DBArenaError::SnapshotError(format!("Failed to start restored container: {}", e))
            })?;

        tracing::info!(
            "Restored snapshot {} to container {}",
            snapshot.id,
            container_name
        );

        // Return Container info
        Ok(crate::container::Container {
            id: response.id,
            name: container_name,
            database_type: snapshot.database_type.to_string(),
            version: "snapshot".to_string(),
            status: crate::container::models::ContainerStatus::Running,
            port: snapshot.database_type.default_port(),
            host_port: if host_port == 0 {
                None
            } else {
                Some(host_port)
            },
            persistent: false,
            created_at: chrono::Utc::now().timestamp(),
        })
    }

    /// Delete a snapshot
    pub async fn delete(&self, snapshot_id: &str) -> Result<()> {
        let snapshot = self.get(snapshot_id).await?;
        self.storage.delete_snapshot(&snapshot).await?;

        tracing::info!("Deleted snapshot {}", snapshot.id);

        Ok(())
    }

    /// Inspect a snapshot (get detailed information)
    pub async fn inspect(&self, snapshot_id: &str) -> Result<Snapshot> {
        self.get(snapshot_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Docker
    async fn test_snapshot_manager_creation() {
        let docker = match Docker::connect_with_local_defaults() {
            Ok(docker) => docker,
            Err(_) => return,
        };
        if docker.ping().await.is_err() {
            return;
        }
        let manager = SnapshotManager::new(Arc::new(docker));

        let result = manager.list().await;
        assert!(result.is_ok());
    }
}
