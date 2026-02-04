use blake3::Hasher;

/// Create a stable doc ID from canonical URL and section path.
pub fn make_doc_id(db: &str, version_slug: &str, canonical_url: &str, section_path: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(canonical_url.as_bytes());
    hasher.update(b"::");
    hasher.update(section_path.as_bytes());
    let hash = hasher.finalize().to_hex();
    let short = &hash[..16];
    format!("{}-{}-{}", db, version_slug, short)
}

/// Parse doc ID into (db, version_slug) if it matches the expected prefix.
pub fn parse_doc_id(doc_id: &str) -> Option<(String, String)> {
    let mut parts = doc_id.splitn(3, '-');
    let db = parts.next()?.to_string();
    let version_slug = parts.next()?.to_string();
    Some((db, version_slug))
}

/// Slugify a database version for consistent filesystem paths and doc IDs.
pub fn slugify_version(version: &str) -> String {
    let mut out = String::new();
    let mut prev_underscore = false;
    for ch in version.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            prev_underscore = false;
        } else if !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    while out.starts_with('_') {
        out.remove(0);
    }
    if out.is_empty() {
        "unknown".to_string()
    } else {
        out
    }
}

/// Slugify a heading for use in anchors.
pub fn slugify_anchor(text: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in text.trim().chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    while out.starts_with('-') {
        out.remove(0);
    }
    if out.is_empty() {
        "section".to_string()
    } else {
        out
    }
}
