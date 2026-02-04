use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::docs::manifest::{DocChunk, DocManifest};
use crate::docs::paths::{pack_content_dir, pack_manifest_path, packs_dir};
use crate::error::{DBArenaError, Result};

pub fn write_chunk(content_dir: &Path, chunk: &DocChunk) -> Result<()> {
    std::fs::create_dir_all(content_dir)?;
    let path = content_dir.join(format!("{}.json", chunk.doc_id));
    let data = serde_json::to_string_pretty(chunk)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn read_chunk(content_dir: &Path, doc_id: &str) -> Result<DocChunk> {
    let path = content_dir.join(format!("{}.json", doc_id));
    let data = std::fs::read_to_string(path)?;
    let chunk: DocChunk = serde_json::from_str(&data)?;
    Ok(chunk)
}

pub fn list_installed_manifests() -> Result<Vec<DocManifest>> {
    let mut manifests = Vec::new();
    let base = packs_dir();
    if !base.exists() {
        return Ok(manifests);
    }
    for entry in WalkDir::new(base).max_depth(3).into_iter().flatten() {
        if entry.file_type().is_file() && entry.file_name() == "manifest.json" {
            let manifest = DocManifest::load(entry.path())?;
            manifests.push(manifest);
        }
    }
    Ok(manifests)
}

pub fn find_pack_by_slug(db: &str, version_slug: &str) -> Result<Option<(DocManifest, PathBuf)>> {
    let base = packs_dir();
    if !base.exists() {
        return Ok(None);
    }
    for entry in WalkDir::new(base).max_depth(3).into_iter().flatten() {
        if entry.file_type().is_file() && entry.file_name() == "manifest.json" {
            let manifest = DocManifest::load(entry.path())?;
            if manifest.db == db && manifest.version_slug == version_slug {
                if let Some(parent) = entry.path().parent() {
                    return Ok(Some((manifest, parent.to_path_buf())));
                }
            }
        }
    }
    Ok(None)
}

pub fn manifest_exists(db: &str, version: &str) -> bool {
    pack_manifest_path(db, version).exists()
}

pub fn load_manifest(db: &str, version: &str) -> Result<DocManifest> {
    DocManifest::load(&pack_manifest_path(db, version))
}

pub fn ensure_pack_dirs(db: &str, version: &str, keep_source: bool) -> Result<()> {
    std::fs::create_dir_all(pack_content_dir(db, version))?;
    std::fs::create_dir_all(pack_manifest_path(db, version).parent().unwrap())?;
    std::fs::create_dir_all(crate::docs::paths::pack_index_dir(db, version))?;
    if keep_source {
        std::fs::create_dir_all(crate::docs::paths::pack_source_dir(db, version))?;
    }
    Ok(())
}

pub fn remove_pack_dir(db: &str, version: &str) -> Result<()> {
    let dir = crate::docs::paths::pack_dir(db, version);
    if dir.exists() {
        std::fs::remove_dir_all(dir)?;
    }
    Ok(())
}

pub fn pack_content_path(db: &str, version: &str) -> PathBuf {
    pack_content_dir(db, version)
}

pub fn pack_index_path(db: &str, version: &str) -> PathBuf {
    crate::docs::paths::pack_index_dir(db, version)
}

pub fn pack_manifest(db: &str, version: &str) -> Result<DocManifest> {
    DocManifest::load(&pack_manifest_path(db, version))
}

pub fn pack_manifest_path_for(db: &str, version: &str) -> PathBuf {
    pack_manifest_path(db, version)
}

pub fn pack_root(db: &str, version: &str) -> PathBuf {
    crate::docs::paths::pack_dir(db, version)
}

pub fn pack_source_path(db: &str, version: &str) -> PathBuf {
    crate::docs::paths::pack_source_dir(db, version)
}

pub fn write_manifest(db: &str, version: &str, manifest: &DocManifest) -> Result<()> {
    manifest.save(&pack_manifest_path(db, version))
}

pub fn ensure_not_installing(db: &str, version: &str) -> Result<()> {
    let lock = pack_root(db, version).join(".installing");
    if lock.exists() {
        return Err(DBArenaError::DocsError(format!(
            "Install already in progress for {} {}",
            db, version
        )));
    }
    Ok(())
}
