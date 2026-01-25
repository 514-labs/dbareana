use crate::cli::interactive;
use crate::container::models::ContainerStatus;
use crate::container::{ContainerManager, DockerClient};
use crate::{DBArenaError, Result};
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

pub async fn handle_stop(
    container: Option<String>,
    interactive_mode: bool,
    all: bool,
    timeout: u64,
) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);

    // Handle --all flag
    if all {
        return handle_stop_all(&manager, timeout).await;
    }

    // Get container name
    let container_name = if interactive_mode {
        // List running containers for selection
        let all_containers = manager.list_containers(false).await?;
        let running_containers: Vec<_> = all_containers
            .into_iter()
            .filter(|c| {
                matches!(
                    c.status,
                    ContainerStatus::Running | ContainerStatus::Healthy
                )
            })
            .collect();

        interactive::select_container(running_containers, "stop")?
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
        "{} Stopping container {}...",
        style("→").cyan(),
        style(&found.name).bold()
    );

    // Stop the container
    manager.stop_container(&found.id, Some(timeout)).await?;
    println!("{} Container stopped", style("✓").green());

    Ok(())
}

async fn handle_stop_all(manager: &ContainerManager, timeout: u64) -> Result<()> {
    // Get all running containers
    let all_containers = manager.list_containers(false).await?;
    let running_containers: Vec<_> = all_containers
        .into_iter()
        .filter(|c| matches!(c.status, ContainerStatus::Running | ContainerStatus::Healthy))
        .collect();

    if running_containers.is_empty() {
        println!("No running containers found.");
        return Ok(());
    }

    let count = running_containers.len();
    println!(
        "{} Stopping {} container(s)...\n",
        style("→").cyan(),
        style(count).bold()
    );

    // Create multi-progress bar
    let multi_progress = MultiProgress::new();
    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .unwrap();

    // Create progress bars for each container
    let progress_bars: Vec<_> = running_containers
        .iter()
        .map(|c| {
            let pb = multi_progress.add(ProgressBar::new_spinner());
            pb.set_style(spinner_style.clone());
            pb.set_message(format!("Stopping {}...", c.name));
            pb.enable_steady_tick(Duration::from_millis(100));
            pb
        })
        .collect();

    // Extract container IDs
    let container_ids: Vec<String> = running_containers.iter().map(|c| c.id.clone()).collect();

    // Stop containers in parallel
    let results = manager.stop_containers_parallel(container_ids, timeout).await;

    // Update progress bars based on results
    let mut success_count = 0;
    let mut failed_count = 0;

    for (i, (container, result)) in running_containers.iter().zip(results.iter()).enumerate() {
        match result {
            Ok(_) => {
                progress_bars[i].finish_with_message(format!(
                    "{} {} stopped",
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
            "Failed to stop {} container(s)",
            failed_count
        )));
    }

    Ok(())
}
