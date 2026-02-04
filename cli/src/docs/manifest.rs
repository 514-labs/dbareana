use serde::{Deserialize, Serialize};

use crate::error::{DBArenaError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub kind: String,
    pub base_url: String,
    pub downloaded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub name: String,
    pub url: String,
    pub accepted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocManifest {
    pub db: String,
    pub version: String,
    pub version_slug: String,
    pub source: SourceInfo,
    pub license: LicenseInfo,
    pub doc_count: usize,
    pub byte_size: u64,
    pub doc_id_scheme: String,
    pub index_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocChunk {
    pub doc_id: String,
    pub title: String,
    pub section_path: String,
    pub body: String,
    pub source_url: String,
}

impl DocManifest {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        serde_json::from_str(&data).map_err(DBArenaError::from)
    }

    pub fn save(&self, path: &std::path::Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.json");
        let manifest = DocManifest {
            db: "postgres".to_string(),
            version: "16".to_string(),
            version_slug: "16".to_string(),
            source: SourceInfo {
                kind: "postgres_html".to_string(),
                base_url: "https://www.postgresql.org/docs/16/".to_string(),
                downloaded_at: "2026-02-04T20:45:00Z".to_string(),
            },
            license: LicenseInfo {
                name: "PostgreSQL Documentation License".to_string(),
                url: "https://www.postgresql.org/about/licence/".to_string(),
                accepted_at: "2026-02-04T20:45:00Z".to_string(),
            },
            doc_count: 1,
            byte_size: 42,
            doc_id_scheme: "blake3(canonical_url + section_path)".to_string(),
            index_version: 1,
        };
        manifest.save(&path).unwrap();
        let loaded = DocManifest::load(&path).unwrap();
        assert_eq!(loaded.db, manifest.db);
        assert_eq!(loaded.version, manifest.version);
        assert_eq!(loaded.doc_count, manifest.doc_count);
    }
}
