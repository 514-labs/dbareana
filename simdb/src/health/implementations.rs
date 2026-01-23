use super::HealthChecker;
use crate::Result;
use async_trait::async_trait;
use bollard::Docker;

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
    async fn check(&self, _container_id: &str) -> Result<bool> {
        // Implementation will be added later
        Ok(true)
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
    async fn check(&self, _container_id: &str) -> Result<bool> {
        // Implementation will be added later
        Ok(true)
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
    async fn check(&self, _container_id: &str) -> Result<bool> {
        // Implementation will be added later
        Ok(true)
    }
}
