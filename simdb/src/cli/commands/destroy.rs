use crate::container::{ContainerManager, DockerClient};
use crate::{Result, SimDbError};
use console::style;
use std::io::{self, Write};

pub async fn handle_destroy(container: String, yes: bool, volumes: bool) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);

    // Find the container
    let found = manager
        .find_container(&container)
        .await?
        .ok_or_else(|| SimDbError::ContainerNotFound(container.clone()))?;

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

    println!(
        "{} Destroying container {}...",
        style("→").cyan(),
        style(&found.name).bold()
    );

    // Destroy the container
    manager.destroy_container(&found.id, volumes).await?;
    println!("{} Container destroyed", style("✓").green());

    Ok(())
}
