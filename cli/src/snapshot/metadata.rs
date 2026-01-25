use serde::{Deserialize, Serialize};
use crate::container::DatabaseType;

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Unique snapshot ID
    pub id: String,
    /// User-friendly snapshot name
    pub name: String,
    /// Source container ID
    pub source_container: String,
    /// Database type
    pub database_type: DatabaseType,
    /// Creation timestamp (Unix timestamp)
    pub created_at: i64,
    /// Docker image tag for this snapshot
    pub image_tag: String,
    /// Optional message describing the snapshot
    pub message: Option<String>,
}

impl Snapshot {
    /// Create a new snapshot metadata
    pub fn new(
        name: String,
        source_container: String,
        database_type: DatabaseType,
        message: Option<String>,
    ) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().timestamp();
        let image_tag = format!("dbarena-snapshot/{}:{}", name, &id[..8]);

        Self {
            id,
            name,
            source_container,
            database_type,
            created_at,
            image_tag,
            message,
        }
    }

    /// Get Docker labels for this snapshot
    pub fn to_labels(&self) -> std::collections::HashMap<String, String> {
        let mut labels = std::collections::HashMap::new();
        labels.insert("dbarena.snapshot".to_string(), "true".to_string());
        labels.insert("dbarena.snapshot.id".to_string(), self.id.clone());
        labels.insert("dbarena.snapshot.name".to_string(), self.name.clone());
        labels.insert(
            "dbarena.snapshot.source".to_string(),
            self.source_container.clone(),
        );
        labels.insert(
            "dbarena.snapshot.database".to_string(),
            self.database_type.to_string(),
        );
        labels.insert(
            "dbarena.snapshot.created_at".to_string(),
            self.created_at.to_string(),
        );
        if let Some(msg) = &self.message {
            labels.insert("dbarena.snapshot.message".to_string(), msg.clone());
        }
        labels
    }

    /// Create snapshot from Docker image labels
    pub fn from_labels(
        _image_id: String,
        image_tag: String,
        labels: &std::collections::HashMap<String, String>,
    ) -> Option<Self> {
        // Check if this is a dbarena snapshot
        if labels.get("dbarena.snapshot")? != "true" {
            return None;
        }

        Some(Self {
            id: labels.get("dbarena.snapshot.id")?.clone(),
            name: labels.get("dbarena.snapshot.name")?.clone(),
            source_container: labels.get("dbarena.snapshot.source")?.clone(),
            database_type: DatabaseType::from_string(
                labels.get("dbarena.snapshot.database")?
            )?,
            created_at: labels
                .get("dbarena.snapshot.created_at")?
                .parse()
                .ok()?,
            image_tag,
            message: labels.get("dbarena.snapshot.message").cloned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let snapshot = Snapshot::new(
            "test-snapshot".to_string(),
            "container-123".to_string(),
            DatabaseType::Postgres,
            Some("Test message".to_string()),
        );

        assert_eq!(snapshot.name, "test-snapshot");
        assert_eq!(snapshot.source_container, "container-123");
        assert!(snapshot.id.len() > 0);
        assert!(snapshot.image_tag.starts_with("dbarena-snapshot/test-snapshot:"));
    }

    #[test]
    fn test_labels_roundtrip() {
        let snapshot = Snapshot::new(
            "test".to_string(),
            "container-123".to_string(),
            DatabaseType::MySQL,
            Some("Test".to_string()),
        );

        let labels = snapshot.to_labels();
        assert_eq!(labels.get("dbarena.snapshot"), Some(&"true".to_string()));
        assert_eq!(
            labels.get("dbarena.snapshot.name"),
            Some(&"test".to_string())
        );
    }
}
