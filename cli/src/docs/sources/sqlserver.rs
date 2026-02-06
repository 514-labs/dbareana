use std::io::Read;
use std::path::Path;

use flate2::read::GzDecoder;

use crate::docs::catalog::DocPack;
use crate::docs::normalize::normalize_markdown;
use crate::docs::sources::NormalizedDoc;
use crate::error::{DBArenaError, Result};

pub async fn fetch_docs(pack: &DocPack, source_dir: Option<&Path>) -> Result<Vec<NormalizedDoc>> {
    let client = reqwest::Client::builder()
        .user_agent("curl/7.79.1")
        .build()
        .map_err(|e| DBArenaError::DocsError(format!("Failed to build HTTP client: {}", e)))?;

    let response = client
        .get(&pack.source_url)
        .send()
        .await
        .map_err(|e| DBArenaError::DocsError(format!("Download failed: {}", e)))?;
    if !response.status().is_success() {
        return Err(DBArenaError::DocsError(format!(
            "Failed to download SQL Server docs: {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| DBArenaError::DocsError(format!("Failed to read response: {}", e)))?;

    if let Some(dir) = source_dir {
        std::fs::create_dir_all(dir)?;
        let path = dir.join("sql-docs.tar.gz");
        let _ = std::fs::write(path, &bytes);
    }

    let decoder = GzDecoder::new(bytes.as_ref());
    let mut archive = tar::Archive::new(decoder);
    let mut docs = Vec::new();

    for entry in archive.entries().map_err(|e| DBArenaError::DocsError(format!("Failed to read tar entries: {}", e)))? {
        let mut entry = entry.map_err(|e| DBArenaError::DocsError(format!("Tar entry error: {}", e)))?;
        let path = entry.path().map_err(|e| DBArenaError::DocsError(format!("Tar path error: {}", e)))?;
        let path_str = path.to_string_lossy().replace('\\', "/");
        if !path_str.contains("/docs/") || !path_str.ends_with(".md") {
            continue;
        }
        let relative = extract_relative_docs_path(&path_str);
        if relative.is_empty() {
            continue;
        }

        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        let content = match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(err) => String::from_utf8_lossy(err.as_bytes()).to_string(),
        };

        if let Some(dir) = source_dir {
            let dest = dir.join(&relative);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let _ = std::fs::write(dest, &content);
        }

        let sections = normalize_markdown(&content);
        let url_path = relative.trim_start_matches('/').trim_end_matches(".md");
        let canonical = format!("{}{}", pack.canonical_base_url, url_path);
        for section in sections {
            docs.push(NormalizedDoc {
                title: section.title,
                section_path: section.section_path,
                body: section.body,
                source_url: canonical.clone(),
            });
        }
    }

    Ok(docs)
}

fn extract_relative_docs_path(path: &str) -> String {
    if let Some(idx) = path.find("/docs/") {
        return path[idx + 6..].to_string();
    }
    String::new()
}
