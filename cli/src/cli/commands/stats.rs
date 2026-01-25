use std::sync::Arc;
use bollard::Docker;
use tokio::time::{sleep, Duration};

use crate::container::{ContainerManager, DockerClient};
use crate::container::models::ContainerStatus;
use crate::error::Result;
use crate::monitoring::{DockerStatsCollector, MetricsCollector, StatsTui};
use crate::monitoring::{display_compact_header, display_metrics_compact, display_metrics_simple};
use crate::config::schema::MonitoringConfig;

/// Handle the stats command
pub async fn handle_stats(
    docker: Arc<Docker>,
    container: Option<String>,
    follow: bool,
    tui: bool,
    all: bool,
    json: bool,
) -> Result<()> {
    let collector = DockerStatsCollector::new(docker.clone());

    // Use default monitoring config interval (500ms for responsive updates)
    // TODO: Load from config file when --config flag is added to stats command
    let monitoring_config = MonitoringConfig::default();
    let interval_ms = monitoring_config.interval_ms;

    if all {
        handle_stats_all(&collector, tui, json, interval_ms).await
    } else {
        let container_id = get_container_id(container).await?;
        handle_stats_single(&collector, &container_id, follow, tui, json, interval_ms).await
    }
}

/// Get container ID from name or interactive selection
async fn get_container_id(container: Option<String>) -> Result<String> {
    if let Some(name) = container {
        // Container name provided - resolve to ID
        let docker_client = DockerClient::new()?;
        let manager = ContainerManager::new(docker_client);
        let found = manager
            .find_container(&name)
            .await?
            .ok_or_else(|| crate::error::DBArenaError::ContainerNotFound(name.clone()))?;
        Ok(found.id)
    } else {
        // Interactive mode - select from running containers
        use crate::cli::interactive::select_container;
        let docker_client = DockerClient::new()?;
        let manager = ContainerManager::new(docker_client);

        let all_containers = manager.list_containers(true).await?;
        let running: Vec<_> = all_containers
            .into_iter()
            .filter(|c| matches!(c.status, ContainerStatus::Running))
            .collect();

        if running.is_empty() {
            return Err(crate::error::DBArenaError::Other(
                "No running containers found".to_string(),
            ));
        }

        let container_name = select_container(running, "monitor")?;

        // Get the ID for the selected container
        let found = manager
            .find_container(&container_name)
            .await?
            .ok_or_else(|| crate::error::DBArenaError::ContainerNotFound(container_name.clone()))?;
        Ok(found.id)
    }
}

/// Handle stats for a single container
async fn handle_stats_single(
    collector: &DockerStatsCollector,
    container_id: &str,
    follow: bool,
    tui: bool,
    json: bool,
    interval_ms: u64,
) -> Result<()> {
    if tui {
        // Launch TUI with configurable interval in milliseconds
        let mut tui_app = StatsTui::new(interval_ms)?;
        tui_app.run_single(collector, container_id).await?;
    } else if follow {
        // Continuous text output
        let mut previous_metrics = None;
        loop {
            let mut metrics = collector.collect(container_id).await?;

            // Calculate rates if we have previous metrics
            if let Some(prev) = &previous_metrics {
                metrics.calculate_rates(prev);
            }

            // Clear screen and display
            print!("\x1B[2J\x1B[1;1H"); // ANSI escape codes to clear screen
            if json {
                println!("{}", serde_json::to_string_pretty(&metrics)?);
            } else {
                display_metrics_simple(&metrics);
            }

            previous_metrics = Some(metrics);
            sleep(Duration::from_millis(interval_ms)).await;
        }
    } else {
        // One-time output
        let metrics = collector.collect(container_id).await?;
        if json {
            println!("{}", serde_json::to_string_pretty(&metrics)?);
        } else {
            display_metrics_simple(&metrics);
        }
    }

    Ok(())
}

/// Handle stats for all containers
async fn handle_stats_all(
    collector: &DockerStatsCollector,
    tui: bool,
    json: bool,
    _interval_ms: u64,
) -> Result<()> {
    if tui {
        // TODO: Implement multi-container TUI dashboard
        return Err(crate::error::DBArenaError::Other(
            "Multi-container TUI not yet implemented. Use without --tui flag for now.".to_string(),
        ));
    }

    if json {
        let metrics = collector.collect_all().await?;
        println!("{}", serde_json::to_string_pretty(&metrics)?);
    } else {
        let metrics = collector.collect_all().await?;
        display_compact_header();
        for m in metrics {
            display_metrics_compact(&m);
        }
    }

    Ok(())
}
