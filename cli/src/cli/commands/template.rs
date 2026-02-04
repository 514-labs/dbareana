use crate::config::{Template, TemplateManager};
use crate::container::{ContainerConfig, ContainerManager, DockerClient, VolumeMount, VolumeMountType};
use crate::{DBArenaError, Result};
use bollard::models::MountPointTypeEnum;
use console::style;
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn handle_template_save(
    container: String,
    name: String,
    description: Option<String>,
) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client.clone());

    // Find the container
    let found = manager
        .find_container(&container)
        .await?
        .ok_or_else(|| DBArenaError::ContainerNotFound(container.clone()))?;

    println!(
        "{} Saving container {} as template {}...",
        style("→").cyan(),
        style(&found.name).bold(),
        style(&name).bold()
    );

    let db_type = crate::container::DatabaseType::from_string(&found.database_type)
        .ok_or_else(|| {
            DBArenaError::InvalidConfig(format!(
                "Unknown database type: {}",
                found.database_type
            ))
        })?;

    let inspect = docker_client
        .docker()
        .inspect_container(&found.id, None)
        .await
        .map_err(|e| {
            DBArenaError::ContainerOperationFailed(format!(
                "Failed to inspect container {}: {}",
                found.id, e
            ))
        })?;

    let labels = inspect
        .config
        .as_ref()
        .and_then(|config| config.labels.as_ref());

    let env_vars = parse_env_vars(
        inspect
            .config
            .as_ref()
            .and_then(|config| config.env.as_ref()),
    );
    let init_scripts = parse_init_scripts(labels);
    let (memory_limit, cpu_shares) = parse_resource_limits(inspect.host_config.as_ref());
    let volumes = parse_volume_mounts(inspect.mounts.as_ref());
    let persistent = !volumes.is_empty();
    let (network, network_aliases) = parse_network_settings(inspect.network_settings.as_ref());
    let host_port = parse_host_port(inspect.network_settings.as_ref(), db_type).or(found.host_port);

    // Create a container config based on the running container
    let config = ContainerConfig {
        name: Some(found.name.clone()),
        database: db_type,
        version: found.version.clone(),
        port: host_port,
        persistent,
        memory_limit,
        cpu_shares,
        env_vars,
        init_scripts: init_scripts.iter().map(PathBuf::from).collect(),
        continue_on_error: false,
        volumes,
    };

    let mut template = Template::from_container_config(name.clone(), description.clone(), &config);
    template.config.network = network;
    template.config.network_aliases = network_aliases;

    let template_manager = TemplateManager::new()?;
    template_manager.save(&template)?;

    println!("  {} Template saved successfully\n", style("✓").green());
    println!("  Name:        {}", template.name);
    println!("  Database:    {}", template.database);
    if let Some(desc) = &template.description {
        println!("  Description: {}", desc);
    }

    Ok(())
}

pub async fn handle_template_list(json: bool) -> Result<()> {
    let template_manager = TemplateManager::new()?;
    let templates = template_manager.list()?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&templates)
                .map_err(|e| DBArenaError::Other(format!("JSON serialization failed: {}", e)))?
        );
        return Ok(());
    }

    if templates.is_empty() {
        println!("No templates found.");
        println!("\nCreate a template with:");
        println!("  dbarena template save <container> --name <template-name>");
        return Ok(());
    }

    // Print header
    println!(
        "{:<25} {:<15} {:<40}",
        "NAME", "DATABASE", "DESCRIPTION"
    );
    println!("{}", "─".repeat(80));

    // Print templates
    for template in templates {
        println!(
            "{:<25} {:<15} {:<40}",
            template.name,
            template.database,
            template
                .description
                .unwrap_or_else(|| "-".to_string())
                .chars()
                .take(40)
                .collect::<String>()
        );
    }

    Ok(())
}

pub async fn handle_template_delete(name: String, yes: bool) -> Result<()> {
    let template_manager = TemplateManager::new()?;

    // Verify template exists
    let _ = template_manager.load(&name)?;

    // Confirm deletion unless --yes flag is used
    if !yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Delete template '{}'?", name))
            .default(false)
            .interact()
            .map_err(|e| DBArenaError::Other(format!("Failed to read confirmation: {}", e)))?;

        if !confirmed {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    println!(
        "{} Deleting template {}...",
        style("→").cyan(),
        style(&name).bold()
    );

    template_manager.delete(&name)?;

    println!("  {} Template deleted successfully", style("✓").green());

    Ok(())
}

pub async fn handle_template_export(name: String, path: PathBuf) -> Result<()> {
    let template_manager = TemplateManager::new()?;

    println!(
        "{} Exporting template {} to {}...",
        style("→").cyan(),
        style(&name).bold(),
        style(path.display()).bold()
    );

    template_manager.export(&name, &path)?;

    println!("  {} Template exported successfully", style("✓").green());

    Ok(())
}

pub async fn handle_template_import(path: PathBuf) -> Result<()> {
    let template_manager = TemplateManager::new()?;

    println!(
        "{} Importing template from {}...",
        style("→").cyan(),
        style(path.display()).bold()
    );

    let template = template_manager.import(&path)?;

    println!("  {} Template imported successfully\n", style("✓").green());
    println!("  Name:     {}", template.name);
    println!("  Database: {}", template.database);

    Ok(())
}

pub async fn handle_template_inspect(name: String, json: bool) -> Result<()> {
    let template_manager = TemplateManager::new()?;
    let template = template_manager.load(&name)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&template)
                .map_err(|e| DBArenaError::Other(format!("JSON serialization failed: {}", e)))?
        );
        return Ok(());
    }

    println!("{} {}", style("Template:").cyan().bold(), style(&template.name).bold());
    println!();
    println!("  Database:    {}", template.database);
    if let Some(desc) = &template.description {
        println!("  Description: {}", desc);
    }

    if let Some(version) = &template.config.version {
        println!("\n  Version:     {}", version);
    }
    if let Some(port) = template.config.port {
        println!("  Port:        {}", port);
    }
    if let Some(persistent) = template.config.persistent {
        println!("  Persistent:  {}", persistent);
    }
    if let Some(memory) = template.config.memory_limit {
        println!("  Memory:      {} MB", memory / (1024 * 1024));
    }

    if !template.config.env_vars.is_empty() {
        println!("\n  Environment Variables:");
        for (key, value) in &template.config.env_vars {
            println!("    {} = {}", key, value);
        }
    }

    if !template.config.init_scripts.is_empty() {
        println!("\n  Init Scripts:");
        for script in &template.config.init_scripts {
            println!("    - {}", script);
        }
    }

    if let Some(network) = &template.config.network {
        println!("\n  Network: {}", network);
        if !template.config.network_aliases.is_empty() {
            println!("  Network Aliases:");
            for alias in &template.config.network_aliases {
                println!("    - {}", alias);
            }
        }
    }

    if !template.config.volumes.is_empty() {
        println!("\n  Volumes:");
        for volume in &template.config.volumes {
            let mount_type = match volume.mount_type {
                VolumeMountType::Volume => "volume",
                VolumeMountType::Bind => "bind",
            };
            let read_only = if volume.read_only { "ro" } else { "rw" };
            println!(
                "    - {} -> {} ({}, {})",
                volume.source, volume.target, mount_type, read_only
            );
        }
    }

    Ok(())
}

fn parse_env_vars(env: Option<&Vec<String>>) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    if let Some(entries) = env {
        for entry in entries {
            if let Some((key, value)) = entry.split_once('=') {
                vars.insert(key.to_string(), value.to_string());
            }
        }
    }
    vars
}

fn parse_init_scripts(labels: Option<&HashMap<String, String>>) -> Vec<String> {
    labels
        .and_then(|map| map.get("dbarena.init_scripts"))
        .and_then(|raw| serde_json::from_str::<Vec<String>>(raw).ok())
        .unwrap_or_default()
}

fn parse_resource_limits(
    host_config: Option<&bollard::models::HostConfig>,
) -> (Option<u64>, Option<u64>) {
    let memory_limit = host_config
        .and_then(|config| config.memory)
        .and_then(|memory| if memory > 0 { Some(memory as u64) } else { None });
    let cpu_shares = host_config
        .and_then(|config| config.cpu_shares)
        .and_then(|cpu| if cpu > 0 { Some(cpu as u64) } else { None });
    (memory_limit, cpu_shares)
}

fn parse_volume_mounts(
    mounts: Option<&Vec<bollard::models::MountPoint>>,
) -> Vec<VolumeMount> {
    let mut volumes = Vec::new();

    if let Some(mounts) = mounts {
        for mount in mounts {
            let mount_type = match mount.typ {
                Some(MountPointTypeEnum::VOLUME) => VolumeMountType::Volume,
                Some(MountPointTypeEnum::BIND) => VolumeMountType::Bind,
                _ => continue,
            };

            let source = match mount_type {
                VolumeMountType::Volume => mount
                    .name
                    .clone()
                    .or_else(|| mount.source.clone()),
                VolumeMountType::Bind => mount.source.clone(),
            };

            let target = mount.destination.clone();

            let (source, target) = match (source, target) {
                (Some(source), Some(target)) => (source, target),
                _ => continue,
            };

            let read_only = mount.rw.map(|rw| !rw).unwrap_or(false);

            volumes.push(VolumeMount {
                source,
                target,
                read_only,
                mount_type,
            });
        }
    }

    volumes
}

fn parse_network_settings(
    network_settings: Option<&bollard::models::NetworkSettings>,
) -> (Option<String>, Vec<String>) {
    let Some(networks) = network_settings.and_then(|settings| settings.networks.as_ref()) else {
        return (None, Vec::new());
    };

    let mut entries: Vec<(&String, &bollard::models::EndpointSettings)> =
        networks.iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));

    if let Some((name, settings)) = entries.first() {
        let aliases = settings.aliases.clone().unwrap_or_default();
        return (Some((*name).clone()), aliases);
    }

    (None, Vec::new())
}

fn parse_host_port(
    network_settings: Option<&bollard::models::NetworkSettings>,
    db_type: crate::container::DatabaseType,
) -> Option<u16> {
    let ports = network_settings.and_then(|settings| settings.ports.as_ref())?;
    let key = format!("{}/tcp", db_type.default_port());
    let bindings = ports.get(&key)?;
    let binding = bindings.as_ref()?.first()?;
    binding.host_port.as_ref()?.parse::<u16>().ok()
}
