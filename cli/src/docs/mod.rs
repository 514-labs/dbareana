pub mod catalog;
pub mod chunk;
pub mod ids;
pub mod index;
pub mod installer;
pub mod manifest;
pub mod normalize;
pub mod paths;
pub mod search;
pub mod sources;
pub mod storage;

pub use catalog::{DocCatalog, DocPack, DocPackSummary, DocSourceKind};
pub use ids::{make_doc_id, parse_doc_id, slugify_anchor, slugify_version};
pub use installer::{install_pack, InstallOptions};
pub use manifest::{DocChunk, DocManifest, LicenseInfo, SourceInfo};
pub use search::{search_pack, SearchResult};
pub use storage::{find_pack_by_slug, list_installed_manifests, read_chunk};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_id_is_stable() {
        let id1 = make_doc_id(
            "postgres",
            "16",
            "https://www.postgresql.org/docs/16/logical-replication.html",
            "Replication → Logical Replication",
        );
        let id2 = make_doc_id(
            "postgres",
            "16",
            "https://www.postgresql.org/docs/16/logical-replication.html",
            "Replication → Logical Replication",
        );
        assert_eq!(id1, id2);
    }
}
