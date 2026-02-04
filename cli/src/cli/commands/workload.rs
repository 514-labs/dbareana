use console::style;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};

use crate::container::{ContainerManager, DatabaseType, DockerClient};
use crate::workload::{print_summary, WorkloadConfig, WorkloadEngine, WorkloadPattern, WorkloadProgressDisplay};
use crate::{DBArenaError, Result};

pub async fn handle_workload_run(
    container: String,
    pattern: Option<String>,
    config: Option<PathBuf>,
    connections: Option<usize>,
    tps: Option<usize>,
    duration: Option<u64>,
    transaction_count: Option<u64>,
) -> Result<()> {
    println!("{}", style("Starting workload...").cyan().bold());
    println!();

    // Find container
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client.clone());
    let container_info = manager
        .find_container(&container)
        .await?
        .ok_or_else(|| DBArenaError::ContainerNotFound(container.clone()))?;

    println!(
        "{} Container: {} ({})",
        style("▸").cyan(),
        style(&container_info.name).green(),
        container_info.database_type
    );

    // Parse database type
    let db_type = DatabaseType::from_string(&container_info.database_type).ok_or_else(|| {
        DBArenaError::InvalidConfig(format!(
            "Unknown database type: {}",
            container_info.database_type
        ))
    })?;

    // Load or create workload config
    let mut workload_config = if let Some(config_path) = config {
        // Load from file
        let config_content = std::fs::read_to_string(&config_path)?;
        toml::from_str(&config_content)
            .map_err(|e| DBArenaError::ConfigError(format!("Failed to parse workload config: {}", e)))?
    } else if let Some(pattern_str) = pattern {
        // Use built-in pattern
        let pattern = WorkloadPattern::from_str(&pattern_str).ok_or_else(|| {
            DBArenaError::InvalidConfig(format!(
                "Unknown pattern: {}. Available: oltp, ecommerce, olap, reporting, time_series, social_media, iot, read_heavy, write_heavy, balanced",
                pattern_str
            ))
        })?;

        println!("{} Pattern: {}", style("▸").cyan(), style(pattern.as_str()).yellow());
        println!("  {}", style(pattern.description()).dim());
        println!();

        // Create config from pattern
        WorkloadConfig {
            name: format!("{:?} Workload", pattern),
            pattern: Some(pattern),
            custom_operations: None,
            custom_queries: None,
            tables: Vec::new(), // Will need to be specified
            connections: connections.unwrap_or(10),
            target_tps: tps.unwrap_or(100),
            duration_seconds: duration,
            transaction_count,
        }
    } else {
        return Err(DBArenaError::InvalidConfig(
            "Must specify either --pattern or --config".to_string(),
        ));
    };

    // Override with CLI parameters
    if let Some(c) = connections {
        workload_config.connections = c;
    }
    if let Some(t) = tps {
        workload_config.target_tps = t;
    }
    if duration.is_some() {
        workload_config.duration_seconds = duration;
    }
    if transaction_count.is_some() {
        workload_config.transaction_count = transaction_count;
    }

    // Validate config
    if workload_config.tables.is_empty() {
        return Err(DBArenaError::InvalidConfig(
            "No tables specified. Use --tables or provide a config file with tables".to_string(),
        ));
    }

    if workload_config.duration_seconds.is_none() && workload_config.transaction_count.is_none() {
        // Default to 60 seconds
        workload_config.duration_seconds = Some(60);
    }

    println!(
        "{} Workers: {}",
        style("▸").cyan(),
        style(workload_config.connections).yellow()
    );
    println!(
        "{} Target TPS: {}",
        style("▸").cyan(),
        style(workload_config.target_tps).yellow()
    );
    println!(
        "{} Tables: {}",
        style("▸").cyan(),
        style(workload_config.tables.join(", ")).yellow()
    );

    if let Some(d) = workload_config.duration_seconds {
        println!("{} Duration: {}s", style("▸").cyan(), style(d).yellow());
    }
    if let Some(c) = workload_config.transaction_count {
        println!("{} Target transactions: {}", style("▸").cyan(), style(c).yellow());
    }

    println!();

    // Create workload engine
    let docker = Arc::new(docker_client.docker().clone());
    let engine = WorkloadEngine::new(
        container_info.id.clone(),
        db_type,
        workload_config.clone(),
        docker,
    );

    // Create progress display
    let progress = WorkloadProgressDisplay::new(
        workload_config.target_tps,
        workload_config.duration_seconds.map(Duration::from_secs),
        workload_config.transaction_count,
    );

    // Start workload in background
    let stats_ref = engine.stats().clone();
    let mut engine_handle = tokio::spawn(async move {
        engine.run().await
    });

    // Show live progress
    let mut progress_interval = interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            _ = progress_interval.tick() => {
                progress.render(&stats_ref);
            }
            result = &mut engine_handle => {
                match result {
                    Ok(Ok(final_stats)) => {
                        // Clear progress display
                        print!("\x1B[2J\x1B[1;1H");

                        // Print final summary
                        let pattern_name = workload_config.pattern
                            .map(|p| p.as_str().to_string())
                            .unwrap_or_else(|| workload_config.name.clone());

                        print_summary(&final_stats, &pattern_name);
                        return Ok(());
                    }
                    Ok(Err(e)) => {
                        return Err(DBArenaError::Other(format!("Workload failed: {}", e)));
                    }
                    Err(e) => {
                        return Err(DBArenaError::Other(format!("Workload task failed: {}", e)));
                    }
                }
            }
        }
    }
}
