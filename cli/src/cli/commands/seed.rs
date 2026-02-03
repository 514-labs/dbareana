use anyhow::anyhow;
use console::style;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crate::container::{ContainerManager, DatabaseType, DockerClient};
use crate::seed::{SeedConfig, SeedStats, SeedingEngine, SizePreset};
use crate::{DBArenaError, Result};

pub async fn handle_seed(
    config_path: PathBuf,
    container: String,
    size: Option<String>,
    seed: Option<u64>,
    truncate: bool,
    _incremental: bool,
    rows_override: Option<String>,
) -> Result<()> {
    let start = Instant::now();

    // Parse configuration
    let config_content = std::fs::read_to_string(&config_path)?;

    let mut config: SeedConfig = toml::from_str(&config_content)
        .map_err(|e| DBArenaError::ConfigError(format!("Failed to parse seed config: {}", e)))?;

    // Apply size preset if specified
    if let Some(size_str) = size {
        let preset = SizePreset::from_str(&size_str).ok_or_else(|| {
            DBArenaError::InvalidConfig(format!(
                "Invalid size preset: {} (use small, medium, or large)",
                size_str
            ))
        })?;

        println!(
            "{} Applying size preset: {}",
            style("▸").cyan(),
            style(preset.as_str()).yellow()
        );

        preset.apply_to_rules(config.seed_rules.tables_mut());
    }

    // Apply row overrides if specified
    if let Some(rows_str) = rows_override {
        apply_row_overrides(&mut config, &rows_str)
            .map_err(|e| DBArenaError::InvalidConfig(e.to_string()))?;
    }

    // Find container
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client.clone());
    let container_info = manager
        .find_container(&container)
        .await?
        .ok_or_else(|| DBArenaError::ContainerNotFound(container.clone()))?;

    println!(
        "{} Seeding container: {} ({})",
        style("▸").cyan(),
        style(&container_info.name).green(),
        container_info.database_type
    );

    // Truncate tables if requested
    if truncate {
        println!("{} Truncating tables...", style("▸").cyan());
        truncate_tables(&manager, &container_info.id, &config)
            .await
            .map_err(|e| DBArenaError::Other(e.to_string()))?;
    }

    // Initialize seeding engine
    let seed_value = seed.or(config.global_seed).unwrap_or(42);
    println!(
        "{} Using seed: {} (for reproducibility)",
        style("▸").cyan(),
        style(seed_value).yellow()
    );

    let db_type = DatabaseType::from_string(&container_info.database_type).ok_or_else(|| {
        DBArenaError::InvalidConfig(format!(
            "Unknown database type: {}",
            container_info.database_type
        ))
    })?;

    let docker = Arc::new(docker_client.docker().clone());
    let mut engine = SeedingEngine::new(
        container_info.id.clone(),
        db_type,
        docker,
        seed_value,
        config.batch_size,
    );

    // Seed all tables
    let rules: Vec<_> = config.seed_rules.tables().to_vec();
    let stats = engine
        .seed_all(&rules)
        .await
        .map_err(|e| DBArenaError::Other(e.to_string()))?;

    // Print summary
    print_summary(&stats, start.elapsed());

    Ok(())
}

fn apply_row_overrides(
    config: &mut SeedConfig,
    overrides: &str,
) -> std::result::Result<(), anyhow::Error> {
    let table_counts: HashMap<String, usize> = overrides
        .split(',')
        .filter_map(|s| {
            let parts: Vec<&str> = s.split('=').collect();
            if parts.len() == 2 {
                let table = parts[0].trim().to_string();
                let count = parts[1].trim().parse::<usize>().ok()?;
                Some((table, count))
            } else {
                None
            }
        })
        .collect();

    if table_counts.is_empty() {
        return Err(anyhow!(
            "Invalid rows override format. Expected: table1=1000,table2=5000"
        ));
    }

    for rule in config.seed_rules.tables_mut() {
        if let Some(&count) = table_counts.get(&rule.name) {
            println!(
                "  {} Overriding {}: {} rows",
                style("→").dim(),
                style(&rule.name).cyan(),
                style(count).yellow()
            );
            rule.count = count;
        }
    }

    Ok(())
}

async fn truncate_tables(
    _manager: &ContainerManager,
    _container_id: &str,
    config: &SeedConfig,
) -> std::result::Result<(), anyhow::Error> {
    for rule in config.seed_rules.tables() {
        println!("  {} Truncating table: {}", style("→").dim(), rule.name);
        // TODO: Implement truncate via Docker exec
        // This will be similar to the execute_sql method in the engine
    }
    Ok(())
}

fn print_summary(stats: &[SeedStats], total_duration: std::time::Duration) {
    println!();
    println!("{}", style("═".repeat(70)).dim());
    println!("{}", style("Seeding Complete").green().bold());
    println!("{}", style("═".repeat(70)).dim());
    println!();

    let mut total_rows = 0;

    for stat in stats {
        total_rows += stat.rows_inserted;
        println!(
            "  {} {}: {} rows in {:.2}s ({:.0} rows/sec)",
            style("✓").green(),
            style(&stat.table).cyan(),
            style(stat.rows_inserted).yellow(),
            stat.duration.as_secs_f64(),
            stat.rows_per_second
        );
    }

    println!();
    println!("{}", style("─".repeat(70)).dim());
    println!(
        "  {} Total: {} rows in {:.2}s ({:.0} rows/sec)",
        style("Σ").cyan().bold(),
        style(total_rows).green().bold(),
        total_duration.as_secs_f64(),
        total_rows as f64 / total_duration.as_secs_f64()
    );
    println!("{}", style("═".repeat(70)).dim());
    println!();
}
