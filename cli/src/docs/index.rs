use std::path::Path;

use tantivy::schema::{FacetOptions, Field, Schema, TextFieldIndexing, TextOptions, STORED, STRING, TEXT};
use tantivy::{Document, Index};

use crate::docs::manifest::DocChunk;
use crate::error::{DBArenaError, Result};

#[derive(Clone, Copy)]
pub struct IndexFields {
    pub doc_id: Field,
    pub db: Field,
    pub version: Field,
    pub title: Field,
    pub section_path: Field,
    pub body: Field,
    pub source_url: Field,
}

pub fn build_index(index_dir: &Path, db: &str, version: &str, chunks: &[DocChunk]) -> Result<()> {
    std::fs::create_dir_all(index_dir)?;
    let (schema, fields) = build_schema();
    let index = Index::create_in_dir(index_dir, schema)
        .map_err(|e| DBArenaError::DocsError(format!("Failed to create index: {}", e)))?;
    let mut writer = index
        .writer(50_000_000)
        .map_err(|e| DBArenaError::DocsError(format!("Failed to create index writer: {}", e)))?;

    for chunk in chunks {
        let mut doc = Document::default();
        doc.add_text(fields.doc_id, &chunk.doc_id);
        doc.add_facet(fields.db, tantivy::schema::Facet::from(&format!("/{}", db)));
        doc.add_facet(fields.version, tantivy::schema::Facet::from(&format!("/{}", version)));
        doc.add_text(fields.title, &chunk.title);
        doc.add_text(fields.section_path, &chunk.section_path);
        doc.add_text(fields.body, &chunk.body);
        doc.add_text(fields.source_url, &chunk.source_url);
        writer.add_document(doc);
    }

    writer
        .commit()
        .map_err(|e| DBArenaError::DocsError(format!("Failed to commit index: {}", e)))?;
    Ok(())
}

pub fn build_schema() -> (Schema, IndexFields) {
    let mut schema_builder = Schema::builder();

    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("default")
        .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default().set_indexing_options(text_indexing).set_stored();

    let doc_id = schema_builder.add_text_field("doc_id", STRING | STORED);
    let db = schema_builder.add_facet_field("db", FacetOptions::default());
    let version = schema_builder.add_facet_field("version", FacetOptions::default());
    let title = schema_builder.add_text_field("title", text_options.clone());
    let section_path = schema_builder.add_text_field("section_path", text_options.clone());
    let body = schema_builder.add_text_field("body", text_options.clone());
    let source_url = schema_builder.add_text_field("source_url", STRING | STORED);

    let schema = schema_builder.build();
    (
        schema,
        IndexFields {
            doc_id,
            db,
            version,
            title,
            section_path,
            body,
            source_url,
        },
    )
}
