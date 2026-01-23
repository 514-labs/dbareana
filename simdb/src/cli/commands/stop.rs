use crate::container::{ContainerManager, DockerClient};
use crate::{Result, SimDbError};
use console::style;

pub async fn handle_stop(container: String, timeout: u64) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);

    // Find the container
    let found = manager
        .find_container(&container)
        .await?
        .ok_or_else(|| SimDbError::ContainerNotFound(container.clone()))?;

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
