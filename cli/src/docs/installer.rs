use std::fs;
use chrono::Utc;
use console::style;

use crate::docs::catalog::{normalize_db_name, DocCatalog, DocSourceKind};
use crate::docs::chunk::chunk_docs;
use crate::docs::index::build_index;
use crate::docs::manifest::{DocManifest, LicenseInfo, SourceInfo};
use crate::docs::paths::{pack_content_dir, pack_index_dir, pack_manifest_path, pack_source_dir, pack_dir};
use crate::docs::sources::{mysql, postgres, sqlserver};
use crate::docs::storage::{ensure_not_installing, write_chunk};
use crate::error::{DBArenaError, Result};

pub struct InstallOptions {
    pub force: bool,
    pub keep_source: bool,
    pub accept_license: bool,
}

pub async fn install_pack(db: &str, version: &str, options: InstallOptions) -> Result<DocManifest> {
    let db_norm = normalize_db_name(db);
    let pack = DocCatalog::get(&db_norm, version).ok_or_else(|| {
        DBArenaError::DocsError(format!(
            "No documentation pack found for {} {}",
            db_norm, version
        ))
    })?;

    let pack_root = pack_dir(&pack.db, &pack.version);
    if pack_root.exists() {
        if !options.force {
            return Err(DBArenaError::DocsError(format!(
                "Pack already installed: {} {}",
                pack.db, pack.version
            )));
        }
        fs::remove_dir_all(&pack_root)?;
    }

    ensure_not_installing(&pack.db, &pack.version)?;
    fs::create_dir_all(&pack_root)?;
    let lock_path = pack_root.join(".installing");
    fs::write(&lock_path, "installing")?;

    let install_result = install_pack_inner(&pack, options).await;
    if install_result.is_err() {
        let _ = fs::remove_dir_all(&pack_root);
    }
    let _ = fs::remove_file(&lock_path);
    install_result
}

async fn install_pack_inner(pack: &crate::docs::catalog::DocPack, options: InstallOptions) -> Result<DocManifest> {
    let content_dir = pack_content_dir(&pack.db, &pack.version);
    let index_dir = pack_index_dir(&pack.db, &pack.version);
    fs::create_dir_all(&content_dir)?;
    fs::create_dir_all(&index_dir)?;

    let source_dir = if options.keep_source {
        let dir = pack_source_dir(&pack.db, &pack.version);
        fs::create_dir_all(&dir)?;
        Some(dir)
    } else {
        None
    };

    let accepted = if options.accept_license {
        true
    } else {
        use dialoguer::Confirm;
        Confirm::new()
            .with_prompt(format!(
                "Accept license '{}' ({}) to install docs?",
                pack.license_name, pack.license_url
            ))
            .interact()
            .map_err(|e| DBArenaError::DocsError(format!("Failed to read input: {}", e)))?
    };

    if !accepted {
        return Err(DBArenaError::DocsError(
            "License not accepted. Aborting install.".to_string(),
        ));
    }

    println!(
        "{} Downloading {} {} docs...",
        style("→").cyan(),
        style(&pack.db).bold(),
        style(&pack.version).dim()
    );

    let docs = match pack.source_kind {
        DocSourceKind::PostgresHtml => {
            postgres::fetch_docs(pack, source_dir.as_deref()).await?
        }
        DocSourceKind::MySqlHtml => mysql::fetch_docs(pack, source_dir.as_deref()).await?,
        DocSourceKind::SqlServerMarkdown => {
            sqlserver::fetch_docs(pack, source_dir.as_deref()).await?
        }
    };

    println!(
        "  {} Normalized {} sections",
        style("✓").green(),
        docs.len()
    );

    let chunks = chunk_docs(&pack.db, &pack.version_slug, docs);
    let mut byte_size: u64 = 0;
    for chunk in &chunks {
        byte_size += chunk.body.len() as u64;
        write_chunk(&content_dir, chunk)?;
    }

    println!(
        "  {} Built {} chunks",
        style("✓").green(),
        chunks.len()
    );

    build_index(&index_dir, &pack.db, &pack.version, &chunks)?;

    println!("  {} Search index created", style("✓").green());

    let now = Utc::now().to_rfc3339();
    let manifest = DocManifest {
        db: pack.db.clone(),
        version: pack.version.clone(),
        version_slug: pack.version_slug.clone(),
        source: SourceInfo {
            kind: pack.source_kind.as_str().to_string(),
            base_url: pack.source_url.clone(),
            downloaded_at: now.clone(),
        },
        license: LicenseInfo {
            name: pack.license_name.clone(),
            url: pack.license_url.clone(),
            accepted_at: now.clone(),
        },
        doc_count: chunks.len(),
        byte_size,
        doc_id_scheme: "blake3(canonical_url + section_path)".to_string(),
        index_version: 1,
    };

    manifest.save(&pack_manifest_path(&pack.db, &pack.version))?;
    Ok(manifest)
}
