use async_trait::async_trait;
use bollard::Docker;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::container::DatabaseType;
use crate::error::Result;

use super::models::DatabaseMetrics;
use super::{mysql, postgres, sqlserver};

/// Trait for collecting database-specific metrics
#[async_trait]
pub trait DatabaseMetricsCollector: Send + Sync {
    async fn collect(&self, container_id: &str, db_type: DatabaseType) -> Result<DatabaseMetrics>;
    async fn supports_database_type(&self, db_type: DatabaseType) -> bool;
}

/// Docker-based database metrics collector using exec commands
pub struct DockerDatabaseMetricsCollector {
    docker_client: Arc<Docker>,
    previous_samples: Arc<Mutex<HashMap<String, DatabaseMetrics>>>,
}

impl DockerDatabaseMetricsCollector {
    pub fn new(docker_client: Arc<Docker>) -> Self {
        Self {
            docker_client,
            previous_samples: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Execute a command in a container and return the output
    pub async fn exec_query(
        &self,
        container_id: &str,
        command: Vec<&str>,
    ) -> Result<String> {
        use bollard::exec::{CreateExecOptions, StartExecResults};
        use futures::StreamExt;

        let exec = self
            .docker_client
            .create_exec(
                container_id,
                CreateExecOptions {
                    cmd: Some(command),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await?;

        let mut output = String::new();
        if let StartExecResults::Attached { output: mut stream, .. } =
            self.docker_client.start_exec(&exec.id, None).await?
        {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(chunk) => {
                        output.push_str(&chunk.to_string());
                    }
                    Err(e) => {
                        return Err(crate::error::DBArenaError::DockerError(e));
                    }
                }
            }
        }

        Ok(output)
    }

    /// Store the current sample for rate calculation
    pub async fn store_sample(&self, metrics: &DatabaseMetrics) {
        let mut samples = self.previous_samples.lock().await;
        samples.insert(metrics.container_id.clone(), metrics.clone());
    }

    /// Get the previous sample for rate calculation
    pub async fn get_previous_sample(&self, container_id: &str) -> Option<DatabaseMetrics> {
        let samples = self.previous_samples.lock().await;
        samples.get(container_id).cloned()
    }
}

#[async_trait]
impl DatabaseMetricsCollector for DockerDatabaseMetricsCollector {
    async fn collect(&self, container_id: &str, db_type: DatabaseType) -> Result<DatabaseMetrics> {
        let metrics = match db_type {
            DatabaseType::Postgres => postgres::collect_metrics(self, container_id).await?,
            DatabaseType::MySQL => mysql::collect_metrics(self, container_id).await?,
            DatabaseType::SQLServer => sqlserver::collect_metrics(self, container_id).await?,
        };

        // Store this sample for next iteration's rate calculation
        self.store_sample(&metrics).await;

        Ok(metrics)
    }

    async fn supports_database_type(&self, _db_type: DatabaseType) -> bool {
        // We support all three database types
        true
    }
}
