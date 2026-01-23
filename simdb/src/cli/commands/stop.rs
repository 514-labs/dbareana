use crate::cli::interactive;
use crate::container::{ContainerManager, DockerClient};
use crate::container::models::ContainerStatus;
use crate::{Result, SimDbError};
use console::style;

pub async fn handle_stop(container: Option<String>, interactive_mode: bool, timeout: u64) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);

    // Get container name
    let container_name = if interactive_mode {
        // List running containers for selection
        let all_containers = manager.list_containers(false).await?;
        let running_containers: Vec<_> = all_containers
            .into_iter()
            .filter(|c| matches!(c.status, ContainerStatus::Running | ContainerStatus::Healthy))
            .collect();

        interactive::select_container(running_containers, "stop")?
    } else {
        container.ok_or_else(|| {
            SimDbError::InvalidConfig(
                "Container name required. Use -i for interactive mode.".to_string(),
            )
        })?
    };

    // Find the container
    let found = manager
        .find_container(&container_name)
        .await?
        .ok_or_else(|| SimDbError::ContainerNotFound(container_name.clone()))?;

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
