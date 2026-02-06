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

        let inspect = self.docker.inspect_container(container_id, None).await?;
        let env_vars = inspect
            .config
            .and_then(|config| config.env)
            .unwrap_or_default();

        let mut user = "postgres".to_string();
        let mut db = "testdb".to_string();

        for env in env_vars {
            if let Some(value) = env.strip_prefix("POSTGRES_USER=") {
                user = value.to_string();
            } else if let Some(value) = env.strip_prefix("POSTGRES_DB=") {
                db = value.to_string();
            }
        }

        let exec = self
            .docker
            .create_exec(
                container_id,
                CreateExecOptions {
                    cmd: Some(vec![
                        "psql".to_string(),
                        "-U".to_string(),
                        user,
                        "-d".to_string(),
                        db,
                        "-t".to_string(),
                        "-A".to_string(),
                        "-c".to_string(),
                        "SELECT 1".to_string(),
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

            let inspect = self.docker.inspect_exec(&exec.id).await?;
            let exit_code = inspect.exit_code.unwrap_or(1);

            let is_healthy = exit_code == 0 && stdout.contains('1');
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

        let inspect = self.docker.inspect_container(container_id, None).await?;
        let env_vars = inspect
            .config
            .and_then(|config| config.env)
            .unwrap_or_default();

        let mut password = "mysql".to_string();
        for env in env_vars {
            if let Some(value) = env.strip_prefix("MYSQL_ROOT_PASSWORD=") {
                password = value.to_string();
            }
        }

        let exec = self
            .docker
            .create_exec(
                container_id,
                CreateExecOptions {
                    cmd: Some(vec![
                        "mysql".to_string(),
                        "-u".to_string(),
                        "root".to_string(),
                        format!("-p{}", password),
                        "--protocol=SOCKET".to_string(),
                        "--connect-timeout=2".to_string(),
                        "-N".to_string(),
                        "-e".to_string(),
                        "SHOW VARIABLES LIKE 'port'; SHOW GLOBAL STATUS LIKE 'Uptime';".to_string(),
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

            let inspect = self.docker.inspect_exec(&exec.id).await?;
            let exit_code = inspect.exit_code.unwrap_or(1);

            let mut uptime_ok = false;
            let mut port_ok = false;

            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if parts[0].eq_ignore_ascii_case("Uptime") {
                        let digits: String = parts[1]
                            .chars()
                            .take_while(|c| c.is_ascii_digit())
                            .collect();
                        if let Ok(uptime) = digits.parse::<u64>() {
                            uptime_ok = uptime >= 5;
                        }
                    } else if parts[0].eq_ignore_ascii_case("port") {
                        let digits: String = parts[1]
                            .chars()
                            .take_while(|c| c.is_ascii_digit())
                            .collect();
                        if let Ok(port) = digits.parse::<u64>() {
                            port_ok = port > 0;
                        }
                    }
                }
            }

            let is_healthy = exit_code == 0 && uptime_ok && port_ok;
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
