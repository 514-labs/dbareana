use std::path::PathBuf;

/// Base directory for docs storage.
pub fn docs_base_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local").join("share").join("dbarena").join("docs");
    }
    PathBuf::from(".").join("dbarena").join("docs")
}

pub fn packs_dir() -> PathBuf {
    docs_base_dir().join("packs")
}

pub fn pack_dir(db: &str, version: &str) -> PathBuf {
    packs_dir().join(db).join(version)
}

pub fn pack_content_dir(db: &str, version: &str) -> PathBuf {
    pack_dir(db, version).join("content")
}

pub fn pack_index_dir(db: &str, version: &str) -> PathBuf {
    pack_dir(db, version).join("index")
}

pub fn pack_source_dir(db: &str, version: &str) -> PathBuf {
    pack_dir(db, version).join("source")
}

pub fn pack_manifest_path(db: &str, version: &str) -> PathBuf {
    pack_dir(db, version).join("manifest.json")
}
