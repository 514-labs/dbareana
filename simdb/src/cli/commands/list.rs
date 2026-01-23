use crate::container::{ContainerManager, DockerClient};
use crate::Result;
use console::style;

pub async fn handle_list(all: bool) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);
    let containers = manager.list_containers(all).await?;

    if containers.is_empty() {
        println!("No containers found.");
        println!(
            "\nCreate a new container with: {}",
            style("simdb create postgres").cyan()
        );
        return Ok(());
    }

    println!("\n{}", style("SimDB Containers").bold());
    println!("{}", "─".repeat(80));
    println!(
        "{:<20} {:<15} {:<12} {:<10} {:<15}",
        "NAME", "DATABASE", "VERSION", "STATUS", "PORT"
    );
    println!("{}", "─".repeat(80));

    for container in containers {
        let status_str = container.status.to_string();
        let status_display = match status_str.as_str() {
            "running" | "healthy" => style(&status_str).green(),
            "stopped" | "exited" => style(&status_str).red(),
            _ => style(&status_str).yellow(),
        };

        let port_display = container
            .host_port
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string());

        println!(
            "{:<20} {:<15} {:<12} {:<10} {:<15}",
            style(&container.name).cyan(),
            container.database_type,
            container.version,
            status_display,
            port_display
        );
    }

    println!();
    Ok(())
}
