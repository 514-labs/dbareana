use bollard::Docker;
use bollard::image::{CommitContainerOptions, ListImagesOptions, RemoveImageOptions};
use std::sync::Arc;

use crate::error::{DBArenaError, Result};
use super::metadata::Snapshot;

/// Storage manager for snapshot images
pub struct SnapshotStorage {
    docker: Arc<Docker>,
}

impl SnapshotStorage {
    pub fn new(docker: Arc<Docker>) -> Self {
        Self { docker }
    }

    /// Commit a container to create a snapshot image
    pub async fn commit_container(
        &self,
        container_id: &str,
        snapshot: &Snapshot,
        pause: bool,
    ) -> Result<()> {
        // Build label changes for Dockerfile format
        // Each LABEL instruction should be a separate string in the Vec
        let label_changes: Vec<String> = snapshot
            .to_labels()
            .iter()
            .map(|(k, v)| format!("LABEL {}=\"{}\"", k, v))
            .collect();

        let config = CommitContainerOptions {
            container: container_id.to_string(),
            repo: snapshot.image_tag.split(':').next().unwrap_or("dbarena-snapshot").to_string(),
            tag: snapshot.image_tag.split(':').nth(1).unwrap_or("latest").to_string(),
            comment: snapshot.message.clone().unwrap_or_default(),
            author: "dbarena".to_string(),
            pause,
            changes: if label_changes.is_empty() {
                None
            } else {
                Some(label_changes.join("\n"))
            },
        };

        self.docker
            .commit_container(config, bollard::container::Config::<String>::default())
            .await
            .map_err(|e| DBArenaError::SnapshotError(format!("Failed to commit container: {}", e)))?;

        Ok(())
    }

    /// List all snapshot images
    pub async fn list_snapshots(&self) -> Result<Vec<Snapshot>> {
        let options = Some(ListImagesOptions::<String> {
            all: true,
            ..Default::default()
        });

        let images = self
            .docker
            .list_images(options)
            .await
            .map_err(|e| DBArenaError::SnapshotError(format!("Failed to list images: {}", e)))?;

        let mut snapshots = Vec::new();
        for image in images {
            let labels = &image.labels;
            let repo_tags = &image.repo_tags;

            for tag in repo_tags {
                if let Some(snapshot) = Snapshot::from_labels(
                    image.id.clone(),
                    tag.clone(),
                    labels,
                ) {
                    snapshots.push(snapshot);
                    break; // Only add once per image
                }
            }
        }

        Ok(snapshots)
    }

    /// Get a specific snapshot by ID
    pub async fn get_snapshot(&self, snapshot_id: &str) -> Result<Option<Snapshot>> {
        let snapshots = self.list_snapshots().await?;
        Ok(snapshots.into_iter().find(|s| s.id == snapshot_id))
    }

    /// Delete a snapshot image
    pub async fn delete_snapshot(&self, snapshot: &Snapshot) -> Result<()> {
        let options = Some(RemoveImageOptions {
            force: true,
            ..Default::default()
        });

        self.docker
            .remove_image(&snapshot.image_tag, options, None)
            .await
            .map_err(|e| DBArenaError::SnapshotError(format!("Failed to remove image: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require Docker to be running
    // They are integration tests and should be run with --ignored flag

    #[tokio::test]
    #[ignore]
    async fn test_list_snapshots() {
        let docker = Docker::connect_with_local_defaults().unwrap();
        let storage = SnapshotStorage::new(Arc::new(docker));

        let result = storage.list_snapshots().await;
        assert!(result.is_ok());
    }
}
