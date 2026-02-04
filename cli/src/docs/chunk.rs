use crate::docs::ids::make_doc_id;
use crate::docs::manifest::DocChunk;
use crate::docs::sources::NormalizedDoc;

const MAX_CHUNK_BYTES: usize = 4096;

pub fn chunk_docs(db: &str, version_slug: &str, docs: Vec<NormalizedDoc>) -> Vec<DocChunk> {
    let mut chunks = Vec::new();
    for doc in docs {
        let body_chunks = split_body(&doc.body);
        let total = body_chunks.len();
        for (idx, body) in body_chunks.into_iter().enumerate() {
            let section_path = if total > 1 {
                format!("{} (Part {})", doc.section_path, idx + 1)
            } else {
                doc.section_path.clone()
            };
            let doc_id = make_doc_id(db, version_slug, &doc.source_url, &section_path);
            chunks.push(DocChunk {
                doc_id,
                title: doc.title.clone(),
                section_path,
                body,
                source_url: doc.source_url.clone(),
            });
        }
    }
    chunks
}

fn split_body(body: &str) -> Vec<String> {
    let paragraphs: Vec<&str> = body
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    if paragraphs.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut current = String::new();
    for para in paragraphs {
        if !current.is_empty() && current.len() + para.len() + 2 > MAX_CHUNK_BYTES {
            chunks.push(current.trim().to_string());
            current.clear();
        }
        if para.len() > MAX_CHUNK_BYTES {
            for piece in split_long(para) {
                if !current.is_empty() {
                    chunks.push(current.trim().to_string());
                    current.clear();
                }
                chunks.push(piece);
            }
            continue;
        }
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(para);
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks
}

fn split_long(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.len() + word.len() + 1 > MAX_CHUNK_BYTES {
            parts.push(current.trim().to_string());
            current.clear();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}
