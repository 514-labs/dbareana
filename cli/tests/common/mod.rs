/// Common test utilities shared across all test modules
use dbarena::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
use dbarena::health::{wait_for_healthy, HealthChecker, MySQLHealthChecker, PostgresHealthChecker, SQLServerHealthChecker};
use std::time::Duration;
use tempfile::TempDir;

pub struct TestContainer {
    pub id: String,
    pub name: String,
    pub manager: ContainerManager,
}

// Note: Automatic cleanup removed due to Clone issues
// Tests should explicitly clean up containers when needed

/// Create a test container with the given configuration
pub async fn create_test_container(
    config: ContainerConfig,
) -> anyhow::Result<TestContainer> {
    let client = DockerClient::new()?;
    client.verify_connection().await?;

    let manager = ContainerManager::new(client);
    let container = manager.create_container(config).await?;

    Ok(TestContainer {
        id: container.id.clone(),
        name: container.name.clone(),
        manager,
    })
}

/// Create and start a test container, waiting for it to be healthy
pub async fn create_and_start_container(
    config: ContainerConfig,
    timeout: Duration,
) -> anyhow::Result<TestContainer> {
    let test_container = create_test_container(config).await?;

    test_container
        .manager
        .start_container(&test_container.id)
        .await?;

    wait_for_healthy_container(&test_container, timeout).await?;

    Ok(test_container)
}

/// Wait for a container to become healthy
pub async fn wait_for_healthy_container(
    test_container: &TestContainer,
    timeout: Duration,
) -> anyhow::Result<()> {
    let client = DockerClient::new()?;
    let docker = client.docker().clone();

    // Determine the health checker based on container name or type
    let container_info = test_container
        .manager
        .find_container(&test_container.id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Container not found"))?;

    let checker: Box<dyn HealthChecker> = match container_info.database_type.as_str() {
        "postgres" => Box::new(PostgresHealthChecker::new(docker)),
        "mysql" => Box::new(MySQLHealthChecker::new(docker)),
        "sqlserver" => Box::new(SQLServerHealthChecker::new(docker)),
        _ => return Err(anyhow::anyhow!("Unknown database type")),
    };

    wait_for_healthy(&test_container.id, checker.as_ref(), timeout).await?;

    Ok(())
}

/// Clean up a container (explicit cleanup, though Drop also handles it)
pub async fn cleanup_container(container_id: &str) -> anyhow::Result<()> {
    let client = DockerClient::new()?;
    let manager = ContainerManager::new(client);
    manager.destroy_container(container_id, false).await?;
    Ok(())
}

/// Execute a query against a running container
/// This is a simplified version - in reality you'd need proper database clients
pub async fn execute_query(
    container_id: &str,
    query: &str,
    database_type: DatabaseType,
) -> anyhow::Result<String> {
    let client = DockerClient::new()?;
    let docker = client.docker();

    let exec_config = match database_type {
        DatabaseType::Postgres => bollard::exec::CreateExecOptions {
            cmd: Some(vec![
                "psql",
                "-U",
                "postgres",
                "-c",
                query,
            ]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        },
        DatabaseType::MySQL => bollard::exec::CreateExecOptions {
            cmd: Some(vec![
                "mysql",
                "-uroot",
                "-ppassword",
                "-e",
                query,
            ]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        },
        DatabaseType::SQLServer => bollard::exec::CreateExecOptions {
            cmd: Some(vec![
                "/opt/mssql-tools/bin/sqlcmd",
                "-S",
                "localhost",
                "-U",
                "sa",
                "-P",
                "YourStrong@Passw0rd",
                "-Q",
                query,
            ]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        },
    };

    let exec = docker.create_exec(container_id, exec_config).await?;

    use futures::StreamExt;
    use bollard::exec::StartExecResults;

    let mut output = String::new();
    if let StartExecResults::Attached { output: mut exec_stream, .. } =
        docker.start_exec(&exec.id, None).await?
    {
        while let Some(Ok(msg)) = exec_stream.next().await {
            output.push_str(&msg.to_string());
        }
    }

    Ok(output)
}

/// Create a temporary directory for test files
pub fn tempdir() -> anyhow::Result<TempDir> {
    Ok(tempfile::tempdir()?)
}

/// Generate a unique test container name
pub fn unique_container_name(prefix: &str) -> String {
    use uuid::Uuid;
    format!("{}-{}", prefix, Uuid::new_v4().to_string()[..8].to_string())
}

/// Check if Docker is available
pub async fn docker_available() -> bool {
    match DockerClient::new() {
        Ok(client) => client.verify_connection().await.is_ok(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_container_name() {
        let name1 = unique_container_name("test");
        let name2 = unique_container_name("test");

        assert_ne!(name1, name2);
        assert!(name1.starts_with("test-"));
        assert!(name2.starts_with("test-"));
    }

    #[test]
    fn test_tempdir() {
        let dir = tempdir().expect("Failed to create tempdir");
        assert!(dir.path().exists());
    }
}
