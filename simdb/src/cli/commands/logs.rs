use crate::container::{ContainerManager, DockerClient};
use crate::{Result, SimDbError};
use bollard::container::LogsOptions;
use futures::StreamExt;

pub async fn handle_logs(container: String, follow: bool, tail: Option<usize>) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);

    // Find the container
    let found = manager
        .find_container(&container)
        .await?
        .ok_or_else(|| SimDbError::ContainerNotFound(container.clone()))?;

    let options = LogsOptions {
        stdout: true,
        stderr: true,
        follow,
        tail: tail.map(|t| t.to_string()).unwrap_or_else(|| "100".to_string()),
        ..Default::default()
    };

    let docker = DockerClient::new()?;
    let mut stream = docker.docker().logs(&found.id, Some(options));

    while let Some(Ok(log)) = stream.next().await {
        print!("{}", log);
    }

    Ok(())
}
