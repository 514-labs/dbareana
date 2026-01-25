use super::models::{Network, NetworkConfig};
use crate::container::DockerClient;
use crate::{DBArenaError, Result};
use bollard::network::{CreateNetworkOptions, InspectNetworkOptions, ListNetworksOptions};
use std::collections::HashMap;
use tracing::{debug, info};

const DBARENA_NETWORK_LABEL: &str = "dbarena.network";

pub struct NetworkManager {
    client: DockerClient,
}

impl NetworkManager {
    pub fn new(client: DockerClient) -> Self {
        Self { client }
    }

    /// Create a new network
    pub async fn create_network(&self, config: NetworkConfig) -> Result<Network> {
        info!("Creating network: {}", config.name);

        // Build labels
        let mut labels = config.labels.clone();
        labels.insert(DBARENA_NETWORK_LABEL.to_string(), "true".to_string());
        labels.insert("dbarena.network.managed".to_string(), "true".to_string());

        // Build IPAM configuration if subnet/gateway specified
        let ipam = if config.subnet.is_some() || config.gateway.is_some() {
            let ipam_config = bollard::models::IpamConfig {
                subnet: config.subnet.clone(),
                gateway: config.gateway.clone(),
                ip_range: None,
                auxiliary_addresses: None,
            };

            Some(bollard::models::Ipam {
                driver: Some("default".to_string()),
                config: Some(vec![ipam_config]),
                options: None,
            })
        } else {
            None
        };

        let mut options = CreateNetworkOptions {
            name: config.name.clone(),
            driver: config.driver.as_str().to_string(),
            internal: config.internal,
            labels: labels.clone(),
            ..Default::default()
        };

        // Set IPAM if configured
        if let Some(ipam_config) = ipam {
            options.ipam = ipam_config;
        }

        let response = self
            .client
            .docker()
            .create_network(options)
            .await
            .map_err(|e| {
                DBArenaError::ContainerOperationFailed(format!("Failed to create network: {}", e))
            })?;

        debug!("Network created with ID: {:?}", response.id);

        Ok(Network {
            id: response.id.unwrap_or_else(|| config.name.clone()),
            name: config.name,
            driver: config.driver.as_str().to_string(),
            subnet: config.subnet,
            gateway: config.gateway,
            internal: config.internal,
            labels,
        })
    }

    /// List networks (optionally filter to dbarena-managed only)
    pub async fn list_networks(&self, managed_only: bool) -> Result<Vec<Network>> {
        let mut filters = HashMap::new();
        if managed_only {
            filters.insert(
                "label".to_string(),
                vec![format!("{}=true", DBARENA_NETWORK_LABEL)],
            );
        }

        let options = ListNetworksOptions {
            filters,
        };

        let networks = self
            .client
            .docker()
            .list_networks(Some(options))
            .await
            .map_err(|e| {
                DBArenaError::ContainerOperationFailed(format!("Failed to list networks: {}", e))
            })?;

        Ok(networks
            .into_iter()
            .filter_map(|n| {
                Some(Network {
                    id: n.id?,
                    name: n.name?,
                    driver: n.driver.unwrap_or_else(|| "unknown".to_string()),
                    subnet: n.ipam.as_ref()
                        .and_then(|ipam| ipam.config.as_ref())
                        .and_then(|configs| configs.first())
                        .and_then(|config| config.subnet.clone()),
                    gateway: n.ipam.as_ref()
                        .and_then(|ipam| ipam.config.as_ref())
                        .and_then(|configs| configs.first())
                        .and_then(|config| config.gateway.clone()),
                    internal: n.internal.unwrap_or(false),
                    labels: n.labels.unwrap_or_default(),
                })
            })
            .collect())
    }

    /// Inspect a specific network
    pub async fn inspect_network(&self, name: &str) -> Result<Network> {
        let options = InspectNetworkOptions {
            verbose: false,
            scope: "local",
        };

        let network = self
            .client
            .docker()
            .inspect_network(name, Some(options))
            .await
            .map_err(|e| {
                DBArenaError::ContainerOperationFailed(format!(
                    "Failed to inspect network {}: {}",
                    name, e
                ))
            })?;

        Ok(Network {
            id: network.id.unwrap_or_else(|| name.to_string()),
            name: network.name.unwrap_or_else(|| name.to_string()),
            driver: network.driver.unwrap_or_else(|| "unknown".to_string()),
            subnet: network.ipam.as_ref()
                .and_then(|ipam| ipam.config.as_ref())
                .and_then(|configs| configs.first())
                .and_then(|config| config.subnet.clone()),
            gateway: network.ipam.as_ref()
                .and_then(|ipam| ipam.config.as_ref())
                .and_then(|configs| configs.first())
                .and_then(|config| config.gateway.clone()),
            internal: network.internal.unwrap_or(false),
            labels: network.labels.unwrap_or_default(),
        })
    }

    /// Delete a network
    pub async fn delete_network(&self, name: &str) -> Result<()> {
        info!("Deleting network: {}", name);

        self.client
            .docker()
            .remove_network(name)
            .await
            .map_err(|e| {
                DBArenaError::ContainerOperationFailed(format!(
                    "Failed to delete network {}: {}",
                    name, e
                ))
            })?;

        debug!("Network deleted: {}", name);
        Ok(())
    }

    /// Connect a container to a network
    pub async fn connect_container(
        &self,
        network: &str,
        container: &str,
        aliases: Option<Vec<String>>,
    ) -> Result<()> {
        info!("Connecting container {} to network {}", container, network);

        let options = bollard::network::ConnectNetworkOptions {
            container,
            endpoint_config: bollard::models::EndpointSettings {
                aliases: aliases.clone(),
                ..Default::default()
            },
        };

        self.client
            .docker()
            .connect_network(network, options)
            .await
            .map_err(|e| {
                DBArenaError::ContainerOperationFailed(format!(
                    "Failed to connect container {} to network {}: {}",
                    container, network, e
                ))
            })?;

        debug!(
            "Container {} connected to network {} with aliases: {:?}",
            container, network, aliases
        );
        Ok(())
    }

    /// Disconnect a container from a network
    pub async fn disconnect_container(&self, network: &str, container: &str) -> Result<()> {
        info!(
            "Disconnecting container {} from network {}",
            container, network
        );

        let options = bollard::network::DisconnectNetworkOptions {
            container,
            force: false,
        };

        self.client
            .docker()
            .disconnect_network(network, options)
            .await
            .map_err(|e| {
                DBArenaError::ContainerOperationFailed(format!(
                    "Failed to disconnect container {} from network {}: {}",
                    container, network, e
                ))
            })?;

        debug!("Container {} disconnected from network {}", container, network);
        Ok(())
    }
}
