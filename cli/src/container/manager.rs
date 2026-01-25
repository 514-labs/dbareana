use super::{Container, ContainerConfig, DockerClient};
use crate::container::models::ContainerStatus;
use crate::Result;
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::models::{ContainerSummary, HostConfig, PortBinding};
use std::collections::HashMap;
use tracing::{debug, info};

const DBARENA_LABEL: &str = "dbarena.managed";

pub struct ContainerManager {
    client: DockerClient,
}

impl ContainerManager {
    pub fn new(client: DockerClient) -> Self {
        Self { client }
    }

    pub async fn create_container(&self, config: ContainerConfig) -> Result<Container> {
        let name = self.generate_container_name(&config);
        let port = config.port.unwrap_or_else(|| self.find_available_port());

        let image = config.database.docker_image(&config.version);

        // Ensure the image exists
        self.client.ensure_image(&image).await?;

        // Build environment variables
        let env = self.build_env_vars(&config);

        // Build port bindings
        let container_port = config.database.default_port();
        let mut port_bindings = HashMap::new();
        port_bindings.insert(
            format!("{}/tcp", container_port),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(port.to_string()),
            }]),
        );

        // Build host config with resource limits
        let mut host_config = HostConfig {
            port_bindings: Some(port_bindings),
            tmpfs: Some(HashMap::from([(
                "/tmp".to_string(),
                "rw,noexec,nosuid,size=256m".to_string(),
            )])),
            ..Default::default()
        };

        if let Some(memory) = config.memory_limit {
            host_config.memory = Some(memory as i64);
        }

        if let Some(cpu_shares) = config.cpu_shares {
            host_config.cpu_shares = Some(cpu_shares as i64);
        }

        // Build labels
        let mut labels = HashMap::new();
        labels.insert(DBARENA_LABEL.to_string(), "true".to_string());
        labels.insert(
            "dbarena.database".to_string(),
            config.database.as_str().to_string(),
        );
        labels.insert("dbarena.version".to_string(), config.version.clone());

        // Create container configuration
        let container_config = Config {
            image: Some(image.clone()),
            env: Some(env),
            labels: Some(labels),
            host_config: Some(host_config),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: name.clone(),
            ..Default::default()
        };

        info!("Creating container: {}", name);
        let response = self
            .client
            .docker()
            .create_container(Some(options), container_config)
            .await?;

        debug!("Container created with ID: {}", response.id);

        Ok(Container {
            id: response.id,
            name: name.clone(),
            database_type: config.database.as_str().to_string(),
            version: config.version,
            status: ContainerStatus::Creating,
            port: container_port,
            host_port: Some(port),
            persistent: config.persistent,
            created_at: chrono::Utc::now().timestamp(),
        })
    }

    pub async fn start_container(&self, id: &str) -> Result<()> {
        info!("Starting container: {}", id);
        self.client
            .docker()
            .start_container(id, None::<StartContainerOptions<String>>)
            .await?;
        Ok(())
    }

    pub async fn stop_container(&self, id: &str, timeout: Option<u64>) -> Result<()> {
        info!("Stopping container: {}", id);
        let options = StopContainerOptions {
            t: timeout.unwrap_or(10) as i64,
        };
        self.client
            .docker()
            .stop_container(id, Some(options))
            .await?;
        Ok(())
    }

    pub async fn destroy_container(&self, id: &str, remove_volumes: bool) -> Result<()> {
        info!("Destroying container: {}", id);
        let options = RemoveContainerOptions {
            v: remove_volumes,
            force: true,
            ..Default::default()
        };
        self.client
            .docker()
            .remove_container(id, Some(options))
            .await?;
        Ok(())
    }

    pub async fn list_containers(&self, all: bool) -> Result<Vec<Container>> {
        let mut filters = HashMap::new();
        filters.insert("label".to_string(), vec![format!("{}=true", DBARENA_LABEL)]);

        let options = ListContainersOptions {
            all,
            filters,
            ..Default::default()
        };

        let containers = self.client.docker().list_containers(Some(options)).await?;

        Ok(containers
            .into_iter()
            .map(|c| self.convert_container(c))
            .collect())
    }

    pub async fn find_container(&self, name_or_id: &str) -> Result<Option<Container>> {
        let containers = self.list_containers(true).await?;
        Ok(containers
            .into_iter()
            .find(|c| c.name == name_or_id || c.id.starts_with(name_or_id)))
    }

    fn generate_container_name(&self, config: &ContainerConfig) -> String {
        if let Some(name) = &config.name {
            name.clone()
        } else {
            let random_suffix: String = rand::random::<u32>().to_string();
            format!(
                "dbarena-{}-{}",
                config.database.as_str(),
                &random_suffix[..6]
            )
        }
    }

    fn find_available_port(&self) -> u16 {
        // Simple random port allocation in the ephemeral range
        // In production, this should check for actual availability
        rand::random::<u16>() % 10000 + 50000
    }

    fn build_env_vars(&self, config: &ContainerConfig) -> Vec<String> {
        // Start with default environment variables for the database type
        let mut env_vars: HashMap<String, String> = match config.database {
            crate::container::DatabaseType::Postgres => HashMap::from([
                ("POSTGRES_PASSWORD".to_string(), "postgres".to_string()),
                ("POSTGRES_USER".to_string(), "postgres".to_string()),
                ("POSTGRES_DB".to_string(), "testdb".to_string()),
            ]),
            crate::container::DatabaseType::MySQL => HashMap::from([
                ("MYSQL_ROOT_PASSWORD".to_string(), "mysql".to_string()),
                ("MYSQL_DATABASE".to_string(), "testdb".to_string()),
            ]),
            crate::container::DatabaseType::SQLServer => HashMap::from([
                ("ACCEPT_EULA".to_string(), "Y".to_string()),
                ("SA_PASSWORD".to_string(), "YourStrong@Passw0rd".to_string()),
            ]),
        };

        // Override with custom environment variables from config
        env_vars.extend(config.env_vars.clone());

        // Convert to Vec<String> format "KEY=VALUE"
        env_vars
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect()
    }

    fn convert_container(&self, summary: ContainerSummary) -> Container {
        let name = summary
            .names
            .as_ref()
            .and_then(|names| names.first())
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let labels = summary.labels.unwrap_or_default();
        let database_type = labels
            .get("dbarena.database")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let version = labels
            .get("dbarena.version")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let status = match summary.state.as_deref() {
            Some("running") => ContainerStatus::Running,
            Some("exited") => ContainerStatus::Exited,
            Some("created") => ContainerStatus::Creating,
            _ => ContainerStatus::Stopped,
        };

        let (port, host_port) = summary
            .ports
            .as_ref()
            .and_then(|ports| ports.first())
            .map(|p| (p.private_port, p.public_port))
            .unwrap_or((0, None));

        Container {
            id: summary.id.unwrap_or_default(),
            name,
            database_type,
            version,
            status,
            port,
            host_port,
            persistent: false,
            created_at: summary.created.unwrap_or(0),
        }
    }

    // Bulk/Parallel Operations

    /// Start multiple containers in parallel
    pub async fn start_containers_parallel(&self, ids: Vec<String>) -> Vec<Result<()>> {
        use futures::future::join_all;
        let futures: Vec<_> = ids.iter().map(|id| {
            let id = id.clone();
            async move { self.start_container(&id).await }
        }).collect();
        join_all(futures).await
    }

    /// Stop multiple containers in parallel
    pub async fn stop_containers_parallel(&self, ids: Vec<String>, timeout: u64) -> Vec<Result<()>> {
        use futures::future::join_all;
        let futures: Vec<_> = ids.iter().map(|id| {
            let id = id.clone();
            async move { self.stop_container(&id, Some(timeout)).await }
        }).collect();
        join_all(futures).await
    }

    /// Destroy multiple containers in parallel
    pub async fn destroy_containers_parallel(
        &self,
        ids: Vec<String>,
        remove_volumes: bool,
    ) -> Vec<Result<()>> {
        use futures::future::join_all;
        let futures: Vec<_> = ids.iter().map(|id| {
            let id = id.clone();
            async move { self.destroy_container(&id, remove_volumes).await }
        }).collect();
        join_all(futures).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::DatabaseType;

    #[test]
    fn test_generate_container_name() {
        let client = DockerClient::new().expect("Failed to create Docker client");
        let manager = ContainerManager::new(client);

        let config = ContainerConfig::new(DatabaseType::Postgres);
        let name = manager.generate_container_name(&config);
        assert!(name.starts_with("dbarena-postgres-"));

        let config_with_name =
            ContainerConfig::new(DatabaseType::MySQL).with_name("my-custom-name".to_string());
        let name = manager.generate_container_name(&config_with_name);
        assert_eq!(name, "my-custom-name");
    }
}
