use crate::cli::interactive;
use crate::container::models::ContainerStatus;
use crate::container::{ContainerManager, DatabaseType, DockerClient};
use crate::health::{
    wait_for_healthy, MySQLHealthChecker, PostgresHealthChecker, SQLServerHealthChecker,
};
use crate::{DBArenaError, Result};
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

const DEFAULT_HEALTH_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn handle_start(container: Option<String>, interactive_mode: bool, all: bool) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);

    // Handle --all flag
    if all {
        return handle_start_all(&manager).await;
    }

    // Get container name
    let container_name = if interactive_mode {
        // List stopped containers for selection
        let all_containers = manager.list_containers(true).await?;
        let stopped_containers: Vec<_> = all_containers
            .into_iter()
            .filter(|c| matches!(c.status, ContainerStatus::Stopped | ContainerStatus::Exited))
            .collect();

        interactive::select_container(stopped_containers, "start")?
    } else {
        container.ok_or_else(|| {
            DBArenaError::InvalidConfig(
                "Container name required. Use -i for interactive mode.".to_string(),
            )
        })?
    };

    // Find the container
    let found = manager
        .find_container(&container_name)
        .await?
        .ok_or_else(|| DBArenaError::ContainerNotFound(container_name.clone()))?;

    println!(
        "{} Starting container {}...",
        style("→").cyan(),
        style(&found.name).bold()
    );

    // Start the container
    manager.start_container(&found.id).await?;
    println!("  {} Container started", style("✓").green());

    // Wait for healthy
    let database = DatabaseType::from_string(&found.database_type).unwrap();
    let checker: Box<dyn crate::health::HealthChecker> = match database {
        DatabaseType::Postgres => Box::new(PostgresHealthChecker::new(
            DockerClient::new()?.docker().clone(),
        )),
        DatabaseType::MySQL => Box::new(MySQLHealthChecker::new(
            DockerClient::new()?.docker().clone(),
        )),
        DatabaseType::SQLServer => Box::new(SQLServerHealthChecker::new(
            DockerClient::new()?.docker().clone(),
        )),
    };

    wait_for_healthy(&found.id, checker.as_ref(), DEFAULT_HEALTH_TIMEOUT).await?;

    println!("\n{}", style("Container is ready!").green().bold());

    Ok(())
}

async fn handle_start_all(manager: &ContainerManager) -> Result<()> {
    // Get all stopped containers
    let all_containers = manager.list_containers(true).await?;
    let stopped_containers: Vec<_> = all_containers
        .into_iter()
        .filter(|c| matches!(c.status, ContainerStatus::Stopped | ContainerStatus::Exited))
        .collect();

    if stopped_containers.is_empty() {
        println!("No stopped containers found.");
        return Ok(());
    }

    let count = stopped_containers.len();
    println!(
        "{} Starting {} container(s)...\n",
        style("→").cyan(),
        style(count).bold()
    );

    // Create multi-progress bar
    let multi_progress = MultiProgress::new();
    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .unwrap();

    // Create progress bars for each container
    let progress_bars: Vec<_> = stopped_containers
        .iter()
        .map(|c| {
            let pb = multi_progress.add(ProgressBar::new_spinner());
            pb.set_style(spinner_style.clone());
            pb.set_message(format!("Starting {}...", c.name));
            pb.enable_steady_tick(Duration::from_millis(100));
            pb
        })
        .collect();

    // Extract container IDs
    let container_ids: Vec<String> = stopped_containers.iter().map(|c| c.id.clone()).collect();

    // Start containers in parallel
    let results = manager.start_containers_parallel(container_ids).await;

    // Update progress bars based on results
    let mut success_count = 0;
    let mut failed_count = 0;

    for (i, (container, result)) in stopped_containers.iter().zip(results.iter()).enumerate() {
        match result {
            Ok(_) => {
                progress_bars[i].finish_with_message(format!(
                    "{} {} started",
                    style("✓").green(),
                    container.name
                ));
                success_count += 1;
            }
            Err(e) => {
                progress_bars[i].finish_with_message(format!(
                    "{} {} failed: {}",
                    style("✗").red(),
                    container.name,
                    e
                ));
                failed_count += 1;
            }
        }
    }

    println!();
    println!(
        "{}",
        style(format!(
            "{} successful, {} failed",
            success_count, failed_count
        ))
        .bold()
    );

    if failed_count > 0 {
        return Err(DBArenaError::Other(format!(
            "Failed to start {} container(s)",
            failed_count
        )));
    }

    Ok(())
}
