use super::HealthChecker;
use crate::Result;
use async_trait::async_trait;
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::Docker;
use futures::StreamExt;
use tracing::debug;

pub struct PostgresHealthChecker {
    docker: Docker,
}

impl PostgresHealthChecker {
    pub fn new(docker: Docker) -> Self {
        Self { docker }
    }
}

#[async_trait]
impl HealthChecker for PostgresHealthChecker {
    async fn check(&self, container_id: &str) -> Result<bool> {
        debug!("Checking PostgreSQL health for container {}", container_id);

        let exec = self
            .docker
            .create_exec(
                container_id,
                CreateExecOptions {
                    cmd: Some(vec!["pg_isready", "-U", "postgres"]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await?;

        if let StartExecResults::Attached { mut output, .. } =
            self.docker.start_exec(&exec.id, None).await?
        {
            let mut stdout = String::new();
            while let Some(Ok(msg)) = output.next().await {
                stdout.push_str(&msg.to_string());
            }

            // pg_isready returns 0 (success) when the database is ready
            let is_healthy = stdout.contains("accepting connections");
            debug!("PostgreSQL health check result: {}", is_healthy);
            Ok(is_healthy)
        } else {
            Ok(false)
        }
    }
}

pub struct MySQLHealthChecker {
    docker: Docker,
}

impl MySQLHealthChecker {
    pub fn new(docker: Docker) -> Self {
        Self { docker }
    }
}

#[async_trait]
impl HealthChecker for MySQLHealthChecker {
    async fn check(&self, container_id: &str) -> Result<bool> {
        debug!("Checking MySQL health for container {}", container_id);

        let exec = self
            .docker
            .create_exec(
                container_id,
                CreateExecOptions {
                    cmd: Some(vec!["mysqladmin", "ping", "-h", "localhost", "-pmysql"]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await?;

        if let StartExecResults::Attached { mut output, .. } =
            self.docker.start_exec(&exec.id, None).await?
        {
            let mut stdout = String::new();
            while let Some(Ok(msg)) = output.next().await {
                stdout.push_str(&msg.to_string());
            }

            // mysqladmin ping returns "mysqld is alive" when ready
            let is_healthy = stdout.contains("mysqld is alive");
            debug!("MySQL health check result: {}", is_healthy);
            Ok(is_healthy)
        } else {
            Ok(false)
        }
    }
}

pub struct SQLServerHealthChecker {
    docker: Docker,
}

impl SQLServerHealthChecker {
    pub fn new(docker: Docker) -> Self {
        Self { docker }
    }
}

#[async_trait]
impl HealthChecker for SQLServerHealthChecker {
    async fn check(&self, container_id: &str) -> Result<bool> {
        debug!("Checking SQL Server health for container {}", container_id);

        let exec = self
            .docker
            .create_exec(
                container_id,
                CreateExecOptions {
                    cmd: Some(vec![
                        "/opt/mssql-tools/bin/sqlcmd",
                        "-S",
                        "localhost",
                        "-U",
                        "sa",
                        "-P",
                        "YourStrong@Passw0rd",
                        "-Q",
                        "SELECT 1",
                    ]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await?;

        if let StartExecResults::Attached { mut output, .. } =
            self.docker.start_exec(&exec.id, None).await?
        {
            let mut stdout = String::new();
            while let Some(Ok(msg)) = output.next().await {
                stdout.push_str(&msg.to_string());
            }

            // If the query succeeds, SQL Server is ready
            let is_healthy = !stdout.contains("Sqlcmd: Error") && !stdout.is_empty();
            debug!("SQL Server health check result: {}", is_healthy);
            Ok(is_healthy)
        } else {
            Ok(false)
        }
    }
}
