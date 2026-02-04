use std::collections::{HashSet, VecDeque};
use std::path::Path;

use reqwest::Url;
use scraper::{Html, Selector};

use crate::docs::catalog::DocPack;
use crate::docs::normalize::normalize_html;
use crate::docs::sources::NormalizedDoc;
use crate::error::{DBArenaError, Result};

const DEFAULT_MAX_PAGES: usize = 1000;

pub async fn fetch_docs(pack: &DocPack, source_dir: Option<&Path>) -> Result<Vec<NormalizedDoc>> {
    let base_url = Url::parse(&pack.source_url)
        .map_err(|e| DBArenaError::DocsError(format!("Invalid base URL: {}", e)))?;
    let client = reqwest::Client::builder()
        .user_agent("curl/7.79.1")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| DBArenaError::DocsError(format!("Failed to build HTTP client: {}", e)))?;

    let max_pages = max_pages();
    let mut queue = VecDeque::new();
    queue.push_back(base_url.clone());
    let mut visited: HashSet<String> = HashSet::new();
    let mut docs = Vec::new();

    while let Some(url) = queue.pop_front() {
        if visited.len() >= max_pages {
            break;
        }
        let url_str = url.to_string();
        if !visited.insert(url_str.clone()) {
            continue;
        }

        let response = client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| DBArenaError::DocsError(format!("Download failed: {}", e)))?;
        if !response.status().is_success() {
            continue;
        }
        let html = response
            .text()
            .await
            .map_err(|e| DBArenaError::DocsError(format!("Failed to read HTML: {}", e)))?;

        if let Some(dir) = source_dir {
            std::fs::create_dir_all(dir)?;
            let file_name = sanitize_filename(url.path());
            let path = dir.join(format!("{}.html", file_name));
            let _ = std::fs::write(path, &html);
        }

        let canonical_url = strip_fragment(&url).to_string();
        let sections = normalize_html(&html);
        for section in sections {
            docs.push(NormalizedDoc {
                title: section.title,
                section_path: section.section_path,
                body: section.body,
                source_url: canonical_url.clone(),
            });
        }

        for link in extract_links(&html, &base_url, &pack.version) {
            let link_str = link.to_string();
            if !visited.contains(&link_str) {
                queue.push_back(link);
            }
        }
    }

    Ok(docs)
}

fn max_pages() -> usize {
    std::env::var("DBARENA_DOCS_MAX_PAGES")
        .ok()
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_PAGES)
}

fn extract_links(html: &str, base_url: &Url, version: &str) -> Vec<Url> {
    let mut links = Vec::new();
    let selector = Selector::parse("a[href]").unwrap();
    let document = Html::parse_document(html);
    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Ok(mut url) = base_url.join(href) {
                url.set_fragment(None);
                url.set_query(None);
                if !is_allowed(&url, base_url, version) {
                    continue;
                }
                links.push(url);
            }
        }
    }
    links
}

fn is_allowed(url: &Url, base_url: &Url, version: &str) -> bool {
    if url.domain() != base_url.domain() {
        return false;
    }
    let path = url.path();
    if !path.contains(&format!("/docs/{}/", version)) {
        return false;
    }
    if path.ends_with(".html") || path.ends_with('/') {
        return true;
    }
    false
}

fn strip_fragment(url: &Url) -> Url {
    let mut clean = url.clone();
    clean.set_fragment(None);
    clean.set_query(None);
    clean
}

fn sanitize_filename(path: &str) -> String {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return "index".to_string();
    }
    trimmed
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}
