use serde::Serialize;

use crate::docs::ids::slugify_version;

#[derive(Debug, Clone, Copy)]
pub enum DocSourceKind {
    PostgresHtml,
    MySqlInfo,
    SqlServerMarkdown,
}

impl DocSourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DocSourceKind::PostgresHtml => "postgres_html",
            DocSourceKind::MySqlInfo => "mysql_info",
            DocSourceKind::SqlServerMarkdown => "sqlserver_markdown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DocPack {
    pub db: String,
    pub version: String,
    pub version_slug: String,
    pub source_kind: DocSourceKind,
    pub source_url: String,
    pub canonical_base_url: String,
    pub license_name: String,
    pub license_url: String,
    pub size_estimate_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocPackSummary {
    pub db: String,
    pub version: String,
    pub version_slug: String,
    pub source_kind: String,
    pub source_url: String,
    pub canonical_base_url: String,
    pub license_name: String,
    pub license_url: String,
    pub size_estimate_bytes: Option<u64>,
}

impl From<&DocPack> for DocPackSummary {
    fn from(pack: &DocPack) -> Self {
        Self {
            db: pack.db.clone(),
            version: pack.version.clone(),
            version_slug: pack.version_slug.clone(),
            source_kind: pack.source_kind.as_str().to_string(),
            source_url: pack.source_url.clone(),
            canonical_base_url: pack.canonical_base_url.clone(),
            license_name: pack.license_name.clone(),
            license_url: pack.license_url.clone(),
            size_estimate_bytes: pack.size_estimate_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DocPackDef {
    db: &'static str,
    version: &'static str,
    source_kind: DocSourceKind,
    source_url: &'static str,
    canonical_base_url: &'static str,
    license_name: &'static str,
    license_url: &'static str,
    size_estimate_bytes: Option<u64>,
}

const PACKS: &[DocPackDef] = &[
    DocPackDef {
        db: "postgres",
        version: "16",
        source_kind: DocSourceKind::PostgresHtml,
        source_url: "https://www.postgresql.org/docs/16/",
        canonical_base_url: "https://www.postgresql.org/docs/16/",
        license_name: "PostgreSQL Documentation License",
        license_url: "https://www.postgresql.org/about/licence/",
        size_estimate_bytes: None,
    },
    DocPackDef {
        db: "postgres",
        version: "15",
        source_kind: DocSourceKind::PostgresHtml,
        source_url: "https://www.postgresql.org/docs/15/",
        canonical_base_url: "https://www.postgresql.org/docs/15/",
        license_name: "PostgreSQL Documentation License",
        license_url: "https://www.postgresql.org/about/licence/",
        size_estimate_bytes: None,
    },
    DocPackDef {
        db: "mysql",
        version: "8.0",
        source_kind: DocSourceKind::MySqlInfo,
        source_url: "https://dev.mysql.com/doc/refman/8.0/en/mysql.info.gz",
        canonical_base_url: "https://dev.mysql.com/doc/refman/8.0/en/",
        license_name: "MySQL Documentation",
        license_url: "https://dev.mysql.com/doc/refman/8.0/en/",
        size_estimate_bytes: None,
    },
    DocPackDef {
        db: "mysql",
        version: "8.4",
        source_kind: DocSourceKind::MySqlInfo,
        source_url: "https://dev.mysql.com/doc/refman/8.4/en/mysql.info.gz",
        canonical_base_url: "https://dev.mysql.com/doc/refman/8.4/en/",
        license_name: "MySQL Documentation",
        license_url: "https://dev.mysql.com/doc/refman/8.4/en/",
        size_estimate_bytes: None,
    },
    DocPackDef {
        db: "sqlserver",
        version: "2022-latest",
        source_kind: DocSourceKind::SqlServerMarkdown,
        source_url: "https://codeload.github.com/MicrosoftDocs/sql-docs/tar.gz/refs/heads/main",
        canonical_base_url: "https://learn.microsoft.com/en-us/sql/",
        license_name: "Microsoft Docs Content License",
        license_url: "https://learn.microsoft.com/en-us/legal/",
        size_estimate_bytes: None,
    },
    DocPackDef {
        db: "sqlserver",
        version: "2019-latest",
        source_kind: DocSourceKind::SqlServerMarkdown,
        source_url: "https://codeload.github.com/MicrosoftDocs/sql-docs/tar.gz/refs/heads/main",
        canonical_base_url: "https://learn.microsoft.com/en-us/sql/",
        license_name: "Microsoft Docs Content License",
        license_url: "https://learn.microsoft.com/en-us/legal/",
        size_estimate_bytes: None,
    },
];

pub struct DocCatalog;

impl DocCatalog {
    pub fn available() -> Vec<DocPack> {
        PACKS
            .iter()
            .map(|def| DocPack {
                db: def.db.to_string(),
                version: def.version.to_string(),
                version_slug: slugify_version(def.version),
                source_kind: def.source_kind,
                source_url: def.source_url.to_string(),
                canonical_base_url: def.canonical_base_url.to_string(),
                license_name: def.license_name.to_string(),
                license_url: def.license_url.to_string(),
                size_estimate_bytes: def.size_estimate_bytes,
            })
            .collect()
    }

    pub fn get(db: &str, version: &str) -> Option<DocPack> {
        let db_norm = normalize_db_name(db);
        PACKS
            .iter()
            .find(|def| def.db == db_norm.as_str() && def.version == version)
            .map(|def| DocPack {
                db: def.db.to_string(),
                version: def.version.to_string(),
                version_slug: slugify_version(def.version),
                source_kind: def.source_kind,
                source_url: def.source_url.to_string(),
                canonical_base_url: def.canonical_base_url.to_string(),
                license_name: def.license_name.to_string(),
                license_url: def.license_url.to_string(),
                size_estimate_bytes: def.size_estimate_bytes,
            })
    }
}

pub fn normalize_db_name(db: &str) -> String {
    let lower = db.to_ascii_lowercase();
    match lower.as_str() {
        "postgresql" => "postgres".to_string(),
        "sql-server" => "sqlserver".to_string(),
        "sql_server" => "sqlserver".to_string(),
        "mssql" => "sqlserver".to_string(),
        _ => lower,
    }
}
