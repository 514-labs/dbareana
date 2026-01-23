use crate::container::{ContainerManager, DatabaseType, DockerClient};
use crate::health::{
    wait_for_healthy, MySQLHealthChecker, PostgresHealthChecker, SQLServerHealthChecker,
};
use crate::{Result, SimDbError};
use console::style;
use std::time::Duration;

const DEFAULT_HEALTH_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn handle_start(container: String) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);

    // Find the container
    let found = manager
        .find_container(&container)
        .await?
        .ok_or_else(|| SimDbError::ContainerNotFound(container.clone()))?;

    println!(
        "{} Starting container {}...",
        style("→").cyan(),
        style(&found.name).bold()
    );

    // Start the container
    manager.start_container(&found.id).await?;
    println!("  {} Container started", style("✓").green());

    // Wait for healthy
    let database = DatabaseType::from_string(&found.database_type).unwrap();
    let checker: Box<dyn crate::health::HealthChecker> = match database {
        DatabaseType::Postgres => Box::new(PostgresHealthChecker::new(
            DockerClient::new()?.docker().clone(),
        )),
        DatabaseType::MySQL => {
            Box::new(MySQLHealthChecker::new(DockerClient::new()?.docker().clone()))
        }
        DatabaseType::SQLServer => Box::new(SQLServerHealthChecker::new(
            DockerClient::new()?.docker().clone(),
        )),
    };

    wait_for_healthy(&found.id, checker.as_ref(), DEFAULT_HEALTH_TIMEOUT).await?;

    println!("\n{}", style("Container is ready!").green().bold());

    Ok(())
}
