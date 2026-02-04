use std::path::Path;

use serde::Serialize;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, QueryParser, TermQuery};
use tantivy::schema::Facet;
use tantivy::{Index, Term};

use crate::docs::index::IndexFields;
use crate::error::{DBArenaError, Result};

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub doc_id: String,
    pub title: String,
    pub section: String,
    pub score: f32,
    pub snippet: String,
    pub source_url: String,
}

pub fn search_pack(
    index_dir: &Path,
    db: &str,
    version: &str,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    let index = Index::open_in_dir(index_dir)
        .map_err(|e| DBArenaError::DocsError(format!("Failed to open index: {}", e)))?;
    let schema = index.schema();
    let fields = extract_fields(&schema)?;

    let reader = index
        .reader()
        .map_err(|e| DBArenaError::DocsError(format!("Failed to open index reader: {}", e)))?;
    let searcher = reader.searcher();

    let mut parser = QueryParser::for_index(&index, vec![fields.title, fields.section_path, fields.body]);
    parser.set_field_boost(fields.title, 2.0);
    parser.set_field_boost(fields.section_path, 1.5);
    parser.set_field_boost(fields.body, 1.0);
    let user_query = parser
        .parse_query(query)
        .map_err(|e| DBArenaError::DocsError(format!("Invalid query: {}", e)))?;

    let db_term = Term::from_facet(fields.db, &Facet::from(&format!("/{}", db)));
    let version_term = Term::from_facet(fields.version, &Facet::from(&format!("/{}", version)));
    let db_query = TermQuery::new(db_term, tantivy::schema::IndexRecordOption::Basic);
    let version_query = TermQuery::new(version_term, tantivy::schema::IndexRecordOption::Basic);

    let boolean_query = BooleanQuery::new(vec![
        (Occur::Must, user_query.clone()),
        (Occur::Must, Box::new(db_query)),
        (Occur::Must, Box::new(version_query)),
    ]);

    let top_docs = searcher
        .search(&boolean_query, &TopDocs::with_limit(limit))
        .map_err(|e| DBArenaError::DocsError(format!("Search failed: {}", e)))?;

    let mut results = Vec::new();
    for (score, doc_address) in top_docs {
        let doc = searcher
            .doc(doc_address)
            .map_err(|e| DBArenaError::DocsError(format!("Failed to load doc: {}", e)))?;

        let doc_id = doc
            .get_first(fields.doc_id)
            .and_then(|v| v.as_text())
            .unwrap_or("")
            .to_string();
        let title = doc
            .get_first(fields.title)
            .and_then(|v| v.as_text())
            .unwrap_or("")
            .to_string();
        let section = doc
            .get_first(fields.section_path)
            .and_then(|v| v.as_text())
            .unwrap_or("")
            .to_string();
        let body = doc
            .get_first(fields.body)
            .and_then(|v| v.as_text())
            .unwrap_or("")
            .to_string();
        let source_url = doc
            .get_first(fields.source_url)
            .and_then(|v| v.as_text())
            .unwrap_or("")
            .to_string();

        let snippet = if body.is_empty() {
            "".to_string()
        } else {
            let mut snippet = body.chars().take(200).collect::<String>();
            if body.len() > snippet.len() {
                snippet.push_str("...");
            }
            snippet
        };

        results.push(SearchResult {
            doc_id,
            title,
            section,
            score,
            snippet,
            source_url,
        });
    }

    Ok(results)
}

fn extract_fields(schema: &tantivy::schema::Schema) -> Result<IndexFields> {
    let doc_id = schema
        .get_field("doc_id")
        .ok_or_else(|| DBArenaError::DocsError("Missing doc_id field".to_string()))?;
    let db = schema
        .get_field("db")
        .ok_or_else(|| DBArenaError::DocsError("Missing db field".to_string()))?;
    let version = schema
        .get_field("version")
        .ok_or_else(|| DBArenaError::DocsError("Missing version field".to_string()))?;
    let title = schema
        .get_field("title")
        .ok_or_else(|| DBArenaError::DocsError("Missing title field".to_string()))?;
    let section_path = schema
        .get_field("section_path")
        .ok_or_else(|| DBArenaError::DocsError("Missing section_path field".to_string()))?;
    let body = schema
        .get_field("body")
        .ok_or_else(|| DBArenaError::DocsError("Missing body field".to_string()))?;
    let source_url = schema
        .get_field("source_url")
        .ok_or_else(|| DBArenaError::DocsError("Missing source_url field".to_string()))?;

    Ok(IndexFields {
        doc_id,
        db,
        version,
        title,
        section_path,
        body,
        source_url,
    })
}
