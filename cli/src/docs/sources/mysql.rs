use std::io::Read;
use std::path::Path;

use flate2::read::GzDecoder;

use crate::docs::catalog::DocPack;
use crate::docs::ids::slugify_anchor;
use crate::docs::normalize::normalize_info;
use crate::docs::sources::NormalizedDoc;
use crate::error::{DBArenaError, Result};

pub async fn fetch_docs(pack: &DocPack, source_dir: Option<&Path>) -> Result<Vec<NormalizedDoc>> {
    let client = reqwest::Client::builder()
        .user_agent("dbarena-docs/0.7.0")
        .build()
        .map_err(|e| DBArenaError::DocsError(format!("Failed to build HTTP client: {}", e)))?;

    let response = client
        .get(&pack.source_url)
        .send()
        .await
        .map_err(|e| DBArenaError::DocsError(format!("Download failed: {}", e)))?;
    if !response.status().is_success() {
        return Err(DBArenaError::DocsError(format!(
            "Failed to download MySQL docs: {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| DBArenaError::DocsError(format!("Failed to read response: {}", e)))?;

    if let Some(dir) = source_dir {
        std::fs::create_dir_all(dir)?;
        let path = dir.join("mysql.info.gz");
        let _ = std::fs::write(path, &bytes);
    }

    let mut decoder = GzDecoder::new(bytes.as_ref());
    let mut info = String::new();
    decoder.read_to_string(&mut info)?;

    let sections = normalize_info(&info);
    let mut docs = Vec::new();
    for section in sections {
        let anchor = slugify_anchor(&section.section_path);
        let canonical = format!("{}#{}", pack.canonical_base_url, anchor);
        docs.push(NormalizedDoc {
            title: section.title,
            section_path: section.section_path,
            body: section.body,
            source_url: canonical,
        });
    }

    Ok(docs)
}
