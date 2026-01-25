use crate::container::DockerClient;
use crate::network::{NetworkConfig, NetworkDriver, NetworkManager};
use crate::{DBArenaError, Result};
use console::style;

pub async fn handle_network_create(
    name: String,
    driver: Option<String>,
    subnet: Option<String>,
    gateway: Option<String>,
    internal: bool,
) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = NetworkManager::new(docker_client);

    // Parse driver
    let network_driver = match driver.as_deref() {
        Some("bridge") | None => NetworkDriver::Bridge,
        Some("host") => NetworkDriver::Host,
        Some("none") => NetworkDriver::None,
        Some(custom) => NetworkDriver::Custom(custom.to_string()),
    };

    let config = NetworkConfig::new(name)
        .with_driver(network_driver)
        .with_internal(internal);

    let config = if let Some(subnet) = subnet {
        config.with_subnet(subnet)
    } else {
        config
    };

    let config = if let Some(gateway) = gateway {
        config.with_gateway(gateway)
    } else {
        config
    };

    println!("{} Creating network {}...", style("→").cyan(), style(&config.name).bold());

    let network = manager.create_network(config).await?;

    println!("  {} Network created successfully\n", style("✓").green());
    println!("  ID:      {}", network.id);
    println!("  Name:    {}", network.name);
    println!("  Driver:  {}", network.driver);
    if let Some(subnet) = network.subnet {
        println!("  Subnet:  {}", subnet);
    }
    if let Some(gateway) = network.gateway {
        println!("  Gateway: {}", gateway);
    }
    if network.internal {
        println!("  Internal: true");
    }

    Ok(())
}

pub async fn handle_network_list(all: bool, json: bool) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = NetworkManager::new(docker_client);
    let networks = manager.list_networks(!all).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&networks)
            .map_err(|e| DBArenaError::Other(format!("JSON serialization failed: {}", e)))?);
        return Ok(());
    }

    if networks.is_empty() {
        if all {
            println!("No networks found.");
        } else {
            println!("No dbarena-managed networks found. Use --all to see all networks.");
        }
        return Ok(());
    }

    // Print header
    println!(
        "{:<25} {:<15} {:<20} {:<15}",
        "NAME", "DRIVER", "SUBNET", "GATEWAY"
    );
    println!("{}", "─".repeat(80));

    // Print networks
    for network in networks {
        println!(
            "{:<25} {:<15} {:<20} {:<15}",
            network.name,
            network.driver,
            network.subnet.as_deref().unwrap_or("-"),
            network.gateway.as_deref().unwrap_or("-")
        );
    }

    Ok(())
}

pub async fn handle_network_inspect(name: String, json: bool) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = NetworkManager::new(docker_client);
    let network = manager.inspect_network(&name).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&network)
            .map_err(|e| DBArenaError::Other(format!("JSON serialization failed: {}", e)))?);
        return Ok(());
    }

    println!("{} {}", style("Network:").cyan().bold(), style(&network.name).bold());
    println!();
    println!("  ID:       {}", network.id);
    println!("  Name:     {}", network.name);
    println!("  Driver:   {}", network.driver);
    if let Some(subnet) = network.subnet {
        println!("  Subnet:   {}", subnet);
    }
    if let Some(gateway) = network.gateway {
        println!("  Gateway:  {}", gateway);
    }
    println!("  Internal: {}", network.internal);

    if !network.labels.is_empty() {
        println!("\n  Labels:");
        for (key, value) in &network.labels {
            println!("    {} = {}", key, value);
        }
    }

    Ok(())
}

pub async fn handle_network_delete(name: String, yes: bool) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = NetworkManager::new(docker_client);

    // Confirm deletion unless --yes flag is used
    if !yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Delete network '{}'?", name))
            .default(false)
            .interact()
            .map_err(|e| DBArenaError::Other(format!("Failed to read confirmation: {}", e)))?;

        if !confirmed {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    println!("{} Deleting network {}...", style("→").cyan(), style(&name).bold());

    manager.delete_network(&name).await?;

    println!("  {} Network deleted successfully", style("✓").green());

    Ok(())
}

pub async fn handle_network_connect(
    network: String,
    container: String,
    aliases: Vec<String>,
) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = NetworkManager::new(docker_client);

    println!(
        "{} Connecting container {} to network {}...",
        style("→").cyan(),
        style(&container).bold(),
        style(&network).bold()
    );

    let aliases_opt = if aliases.is_empty() {
        None
    } else {
        Some(aliases.clone())
    };

    manager.connect_container(&network, &container, aliases_opt).await?;

    println!("  {} Container connected successfully", style("✓").green());
    if !aliases.is_empty() {
        println!("  Aliases: {}", aliases.join(", "));
    }

    Ok(())
}

pub async fn handle_network_disconnect(network: String, container: String) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = NetworkManager::new(docker_client);

    println!(
        "{} Disconnecting container {} from network {}...",
        style("→").cyan(),
        style(&container).bold(),
        style(&network).bold()
    );

    manager.disconnect_container(&network, &container).await?;

    println!("  {} Container disconnected successfully", style("✓").green());

    Ok(())
}
