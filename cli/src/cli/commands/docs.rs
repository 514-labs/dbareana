use console::style;

use crate::docs::catalog::{normalize_db_name, DocCatalog, DocPackSummary};
use crate::docs::installer::{install_pack, InstallOptions};
use crate::docs::paths::{pack_index_dir, pack_dir};
use crate::docs::search::search_pack;
use crate::docs::storage::{find_pack_by_slug, list_installed_manifests, read_chunk};
use crate::docs::{parse_doc_id};
use crate::error::{DBArenaError, Result};

/// Handle docs list command
pub async fn handle_docs_list(installed_only: bool, available_only: bool, json: bool) -> Result<()> {
    let mut installed_only = installed_only;
    let mut available_only = available_only;
    if installed_only && available_only {
        installed_only = false;
        available_only = false;
    }
    let available = DocCatalog::available();
    let installed = list_installed_manifests()?;

    if json {
        let available_summaries: Vec<DocPackSummary> =
            available.iter().map(DocPackSummary::from).collect();
        let output = if installed_only {
            serde_json::json!({ "installed": installed })
        } else if available_only {
            serde_json::json!({ "available": available_summaries })
        } else {
            serde_json::json!({ "available": available_summaries, "installed": installed })
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if !installed_only {
        println!("{}", style("Available Doc Packs").cyan().bold());
        println!("{}", "─".repeat(60));
        for pack in available {
            println!(
                "  {:<10} {:<12} {}",
                pack.db,
                pack.version,
                pack.source_kind.as_str()
            );
        }
        println!();
    }

    if !available_only {
        println!("{}", style("Installed Doc Packs").cyan().bold());
        println!("{}", "─".repeat(60));
        if installed.is_empty() {
            println!("{}", style("No installed doc packs.").yellow());
        } else {
            for pack in installed {
                println!(
                    "  {:<10} {:<12} {} docs",
                    pack.db, pack.version, pack.doc_count
                );
            }
        }
    }

    Ok(())
}

/// Handle docs install command
pub async fn handle_docs_install(
    db: String,
    version: String,
    force: bool,
    keep_source: bool,
    accept_license: bool,
) -> Result<()> {
    let db = normalize_db_name(&db);
    let options = InstallOptions {
        force,
        keep_source,
        accept_license,
    };
    let manifest = install_pack(&db, &version, options).await?;

    println!("  {} Pack installed successfully", style("✓").green());
    println!();
    println!("  DB:          {}", manifest.db);
    println!("  Version:     {}", manifest.version);
    println!("  Doc Count:   {}", manifest.doc_count);
    println!("  Byte Size:   {}", manifest.byte_size);

    Ok(())
}

/// Handle docs search command
pub async fn handle_docs_search(
    db: String,
    version: String,
    query: String,
    limit: usize,
    json: bool,
) -> Result<()> {
    let db = normalize_db_name(&db);
    let index_dir = pack_index_dir(&db, &version);
    if !index_dir.exists() {
        return Err(DBArenaError::DocsError(format!(
            "Docs pack not installed for {} {}",
            db, version
        )));
    }
    let results = search_pack(&index_dir, &db, &version, &query, limit)?;
    if json {
        let output = serde_json::json!({
            "query": query,
            "db": db,
            "version": version,
            "results": results,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if results.is_empty() {
        println!("{}", style("No results found.").yellow());
        return Ok(());
    }

    for (idx, result) in results.iter().enumerate() {
        println!("{}. {} ({})", idx + 1, result.title, result.section);
        println!("   Score: {:.2}", result.score);
        println!("   URL:   {}", result.source_url);
        println!("   {}", result.snippet);
        println!();
    }

    Ok(())
}

/// Handle docs show command
pub async fn handle_docs_show(doc_id: String, max_chars: usize, json: bool) -> Result<()> {
    let (db, version_slug) = parse_doc_id(&doc_id).ok_or_else(|| {
        DBArenaError::DocsError("Invalid doc_id format".to_string())
    })?;
    let found = find_pack_by_slug(&db, &version_slug)?;
    let (_manifest, pack_root) = found.ok_or_else(|| {
        DBArenaError::DocsError(format!(
            "No installed pack found for {} {}",
            db, version_slug
        ))
    })?;

    let content_dir = pack_root.join("content");
    let mut chunk = read_chunk(&content_dir, &doc_id)?;
    if chunk.body.len() > max_chars {
        chunk.body = chunk.body.chars().take(max_chars).collect::<String>() + "...";
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&chunk)?);
        return Ok(());
    }

    println!("{}", style("Documentation Chunk").cyan().bold());
    println!("{}", "─".repeat(60));
    println!("Title:   {}", chunk.title);
    println!("Section: {}", chunk.section_path);
    println!("Source:  {}", chunk.source_url);
    println!();
    println!("{}", chunk.body);

    Ok(())
}

/// Handle docs remove command
pub async fn handle_docs_remove(db: String, version: String, yes: bool) -> Result<()> {
    let db = normalize_db_name(&db);
    let dir = pack_dir(&db, &version);
    if !dir.exists() {
        return Err(DBArenaError::DocsError(format!(
            "Docs pack not installed for {} {}",
            db, version
        )));
    }

    if !yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!(
                "Remove docs pack for {} {}? This will delete local files.",
                db, version
            ))
            .interact()
            .map_err(|e| DBArenaError::DocsError(format!("Failed to read input: {}", e)))?;

        if !confirmed {
            println!("{}", style("Cancelled.").yellow());
            return Ok(());
        }
    }

    std::fs::remove_dir_all(dir)?;
    println!("  {} Docs pack removed", style("✓").green());
    Ok(())
}
