pub mod mysql;
pub mod postgres;
pub mod sqlserver;

#[derive(Debug, Clone)]
pub struct NormalizedDoc {
    pub title: String,
    pub section_path: String,
    pub body: String,
    pub source_url: String,
}
