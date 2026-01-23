use crate::cli::interactive;
use crate::container::{ContainerManager, DockerClient};
use crate::{Result, SimDbError};
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::io::{self, Write};
use std::time::Instant;

pub async fn handle_destroy(
    container: Option<String>,
    interactive_mode: bool,
    yes: bool,
    volumes: bool,
) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);

    // Get container names
    let container_names = if interactive_mode {
        // List all containers for selection (multi-select)
        let all_containers = manager.list_containers(true).await?;
        interactive::select_containers(all_containers, "destroy")?
    } else {
        vec![container.ok_or_else(|| {
            SimDbError::InvalidConfig(
                "Container name required. Use -i for interactive mode.".to_string(),
            )
        })?]
    };

    // If multiple containers, use multi-progress
    if container_names.len() > 1 {
        destroy_multiple_with_progress(&manager, container_names, yes, volumes).await
    } else {
        destroy_single(&manager, container_names.into_iter().next().unwrap(), yes, volumes).await
    }
}

async fn destroy_single(
    manager: &ContainerManager,
    container_name: String,
    yes: bool,
    volumes: bool,
) -> Result<()> {
    // Find the container
    let found = manager
        .find_container(&container_name)
        .await?
        .ok_or_else(|| SimDbError::ContainerNotFound(container_name.clone()))?;

    // Confirmation prompt if not using -y flag
    if !yes {
        print!(
            "Are you sure you want to destroy container {}? [y/N] ",
            style(&found.name).bold()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.red} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Destroying {}...", found.name));

    // Destroy the container
    manager.destroy_container(&found.id, volumes).await?;
    pb.finish_with_message(format!("{} {} destroyed", style("✓").green(), found.name));

    Ok(())
}

async fn destroy_multiple_with_progress(
    manager: &ContainerManager,
    container_names: Vec<String>,
    yes: bool,
    volumes: bool,
) -> Result<()> {
    println!("\n{}", style("Destroying containers...").bold().red());
    println!("{}", "─".repeat(80));

    let start_time = Instant::now();
    let multi_progress = MultiProgress::new();
    let mut confirmed_containers = Vec::new();

    // Collect confirmed containers
    for container_name in container_names {
        let found = manager
            .find_container(&container_name)
            .await?
            .ok_or_else(|| SimDbError::ContainerNotFound(container_name.clone()))?;

        // Confirmation prompt if not using -y flag
        if !yes {
            print!(
                "Destroy container {}? [y/N] ",
                style(&found.name).bold()
            );
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Skipped {}.", found.name);
                continue;
            }
        }

        confirmed_containers.push(found);
    }

    if confirmed_containers.is_empty() {
        println!("No containers to destroy.");
        return Ok(());
    }

    println!();

    // Destroy all confirmed containers sequentially with progress bars
    let mut success_count = 0;
    let mut failed_count = 0;

    for container in confirmed_containers {
        let container_id = container.id.clone();
        let container_name = container.name.clone();

        let pb = multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.red} {msg}")
                .unwrap(),
        );
        pb.set_message(format!("Destroying {:<20}", container_name));

        match manager.destroy_container(&container_id, volumes).await {
            Ok(()) => {
                pb.finish_with_message(format!("{} {:<20} destroyed", style("✓").green(), container_name));
                success_count += 1;
            }
            Err(e) => {
                pb.finish_with_message(format!("{} {:<20} failed: {}", style("✗").red(), container_name, e));
                failed_count += 1;
            }
        }
    }

    let elapsed = start_time.elapsed();

    // Print summary
    println!("\n{}", "─".repeat(80));
    println!("{} {} successful, {} failed in {:.2}s",
        style("Summary:").bold(),
        style(success_count).green(),
        style(failed_count).red(),
        elapsed.as_secs_f64()
    );

    if failed_count > 0 {
        return Err(SimDbError::Other(format!("{} container(s) failed to destroy", failed_count)));
    }

    Ok(())
}
