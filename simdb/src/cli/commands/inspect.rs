use crate::cli::interactive;
use crate::container::{ContainerManager, DockerClient};
use crate::{DBArenaError, Result};
use console::style;

pub async fn handle_inspect(container: Option<String>, interactive_mode: bool) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);

    // Get container name
    let container_name = if interactive_mode {
        // List all containers for selection
        let all_containers = manager.list_containers(true).await?;
        interactive::select_container(all_containers, "inspect")?
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

    println!("\n{}", style("Container Details").bold());
    println!("{}", "─".repeat(50));
    println!("  {}: {}", style("Name").bold(), found.name);
    println!("  {}: {}", style("ID").bold(), &found.id[..12]);
    println!("  {}: {}", style("Database").bold(), found.database_type);
    println!("  {}: {}", style("Version").bold(), found.version);
    println!("  {}: {}", style("Status").bold(), found.status);
    println!(
        "  {}: {}",
        style("Port").bold(),
        found
            .host_port
            .map(|p| p.to_string())
            .unwrap_or_else(|| "N/A".to_string())
    );
    println!(
        "  {}: {}",
        style("Persistent").bold(),
        if found.persistent { "Yes" } else { "No" }
    );

    // Convert timestamp to readable date
    let created_date = chrono::DateTime::from_timestamp(found.created_at, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    println!("  {}: {}", style("Created").bold(), created_date);

    println!();
    Ok(())
}
