/// Common test utilities shared across all test modules
use dbarena::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
use dbarena::health::{wait_for_healthy, HealthChecker, MySQLHealthChecker, PostgresHealthChecker, SQLServerHealthChecker};
use std::net::TcpListener;
use std::time::Duration;
use tempfile::TempDir;

pub struct TestContainer {
    pub id: String,
    pub name: String,
    pub manager: ContainerManager,
}

impl Drop for TestContainer {
    fn drop(&mut self) {
        let id = self.id.clone();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                if let Ok(client) = DockerClient::new() {
                    let manager = ContainerManager::new(client);
                    let _ = manager.destroy_container(&id, false).await;
                }
            });
        } else {
            std::thread::spawn(move || {
                if let Ok(rt) = tokio::runtime::Runtime::new() {
                    rt.block_on(async move {
                        if let Ok(client) = DockerClient::new() {
                            let manager = ContainerManager::new(client);
                            let _ = manager.destroy_container(&id, false).await;
                        }
                    });
                }
            });
        }
    }
}

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
    let db_type = config.database;
    let test_container = create_test_container(config).await?;

    test_container
        .manager
        .start_container(&test_container.id)
        .await?;

    let effective_timeout = match db_type {
        DatabaseType::MySQL => std::cmp::max(timeout, Duration::from_secs(180)),
        DatabaseType::Postgres => std::cmp::max(timeout, Duration::from_secs(120)),
        DatabaseType::SQLServer => std::cmp::max(timeout, Duration::from_secs(180)),
    };

    wait_for_healthy_container(&test_container, effective_timeout).await?;

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

    let inspect = docker.inspect_container(container_id, None).await?;
    let env_vars = inspect
        .config
        .and_then(|config| config.env)
        .unwrap_or_default();

    let mut env_map = std::collections::HashMap::new();
    for env in env_vars {
        if let Some((key, value)) = env.split_once('=') {
            env_map.insert(key.to_string(), value.to_string());
        }
    }

    let exec_config = match database_type {
        DatabaseType::Postgres => bollard::exec::CreateExecOptions {
            cmd: Some(vec![
                "psql".to_string(),
                "-U".to_string(),
                env_map
                    .get("POSTGRES_USER")
                    .cloned()
                    .unwrap_or_else(|| "postgres".to_string()),
                "-d".to_string(),
                env_map
                    .get("POSTGRES_DB")
                    .cloned()
                    .unwrap_or_else(|| "testdb".to_string()),
                "-v".to_string(),
                "ON_ERROR_STOP=1".to_string(),
                "-t".to_string(),
                "-A".to_string(),
                "-c".to_string(),
                query.to_string(),
            ]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        },
        DatabaseType::MySQL => bollard::exec::CreateExecOptions {
            cmd: Some(vec![
                "mysql".to_string(),
                "-uroot".to_string(),
                env_map
                    .get("MYSQL_ROOT_PASSWORD")
                    .map(|value| format!("-p{}", value))
                    .unwrap_or_else(|| "-pmysql".to_string()),
                "-D".to_string(),
                env_map
                    .get("MYSQL_DATABASE")
                    .cloned()
                    .unwrap_or_else(|| "testdb".to_string()),
                "-e".to_string(),
                query.to_string(),
            ]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        },
        DatabaseType::SQLServer => bollard::exec::CreateExecOptions {
            cmd: Some(vec![
                "/opt/mssql-tools/bin/sqlcmd".to_string(),
                "-S".to_string(),
                "localhost".to_string(),
                "-U".to_string(),
                "sa".to_string(),
                "-P".to_string(),
                env_map
                    .get("SA_PASSWORD")
                    .cloned()
                    .unwrap_or_else(|| "YourStrong@Passw0rd".to_string()),
                "-b".to_string(),
                "-Q".to_string(),
                query.to_string(),
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

    let inspect = docker.inspect_exec(&exec.id).await?;
    let exit_code = inspect.exit_code.unwrap_or(1);
    if exit_code != 0 {
        return Err(anyhow::anyhow!(
            "Command failed with exit code {}: {}",
            exit_code,
            output
        ));
    }

    Ok(output)
}

/// Create a temporary directory for test files
pub fn tempdir() -> anyhow::Result<TempDir> {
    Ok(tempfile::tempdir()?)
}

/// Find a free local port for binding.
pub fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to local port");
    listener.local_addr().expect("Failed to read local addr").port()
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
