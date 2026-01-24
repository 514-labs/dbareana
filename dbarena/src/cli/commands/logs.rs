use crate::cli::interactive;
use crate::container::{ContainerManager, DockerClient};
use crate::{DBArenaError, Result};
use bollard::container::LogsOptions;
use futures::StreamExt;

pub async fn handle_logs(
    container: Option<String>,
    interactive_mode: bool,
    follow: bool,
    tail: Option<usize>,
) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);

    // Get container name
    let container_name = if interactive_mode {
        // List all containers for selection
        let all_containers = manager.list_containers(true).await?;
        interactive::select_container(all_containers, "view logs")?
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

    let options = LogsOptions {
        stdout: true,
        stderr: true,
        follow,
        tail: tail
            .map(|t| t.to_string())
            .unwrap_or_else(|| "100".to_string()),
        ..Default::default()
    };

    let docker = DockerClient::new()?;
    let mut stream = docker.docker().logs(&found.id, Some(options));

    while let Some(Ok(log)) = stream.next().await {
        print!("{}", log);
    }

    Ok(())
}
