use crate::cli::interactive;
use crate::container::{ContainerManager, DockerClient};
use crate::{Result, SimDbError};
use console::style;
use std::io::{self, Write};

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

    // Process each container
    for container_name in container_names {
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
                println!("Skipped {}.", found.name);
                continue;
            }
        }

        println!(
            "{} Destroying container {}...",
            style("→").cyan(),
            style(&found.name).bold()
        );

        // Destroy the container
        manager.destroy_container(&found.id, volumes).await?;
        println!("{} Container destroyed", style("✓").green());
    }

    Ok(())
}
