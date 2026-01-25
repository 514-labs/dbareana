use std::sync::Arc;
use bollard::Docker;
use console::style;

use crate::error::{DBArenaError, Result};
use crate::container::{VolumeManager, VolumeConfig};

/// Handle volume create command
pub async fn handle_volume_create(
    name: String,
    mount_path: Option<String>,
) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|_| DBArenaError::DockerNotAvailable)?;
    let manager = VolumeManager::new(Arc::new(docker));

    let config = VolumeConfig::new(
        name.clone(),
        mount_path.unwrap_or_else(|| "/data".to_string()),
    );

    println!(
        "{} Creating volume {}...",
        style("→").cyan(),
        style(&name).bold()
    );

    let volume_name = manager.create(config).await?;

    println!("  {} Volume created successfully", style("✓").green());
    println!();
    println!("  Name: {}", volume_name);

    Ok(())
}

/// Handle volume list command
pub async fn handle_volume_list(all: bool, json: bool) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|_| DBArenaError::DockerNotAvailable)?;
    let manager = VolumeManager::new(Arc::new(docker));

    let volumes = manager.list(!all).await?; // If not --all, show only managed

    if volumes.is_empty() {
        if !json {
            if all {
                println!("{}", style("No volumes found.").yellow());
            } else {
                println!("{}", style("No dbarena-managed volumes found. Use --all to see all volumes.").yellow());
            }
        } else {
            println!("[]");
        }
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&volumes)?);
    } else {
        println!(
            "{:<30} {:<15} {:<50}",
            "NAME", "DRIVER", "MOUNTPOINT"
        );
        println!("{}", "─".repeat(95));

        for volume in volumes {
            println!(
                "{:<30} {:<15} {:<50}",
                truncate_string(&volume.name, 30),
                volume.driver,
                truncate_string(&volume.mountpoint, 50)
            );
        }
    }

    Ok(())
}

/// Handle volume delete command
pub async fn handle_volume_delete(name: String, force: bool, yes: bool) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|_| DBArenaError::DockerNotAvailable)?;
    let manager = VolumeManager::new(Arc::new(docker));

    if !yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!(
                "Delete volume '{}'? This cannot be undone and may cause data loss.",
                name
            ))
            .interact()
            .map_err(|e| DBArenaError::Other(format!("Failed to read input: {}", e)))?;

        if !confirmed {
            println!("{}", style("Cancelled.").yellow());
            return Ok(());
        }
    }

    println!(
        "{} Deleting volume {}...",
        style("→").cyan(),
        style(&name).bold()
    );

    manager.delete(&name, force).await?;

    println!("  {} Volume deleted successfully", style("✓").green());

    Ok(())
}

/// Handle volume inspect command
pub async fn handle_volume_inspect(name: String, json: bool) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|_| DBArenaError::DockerNotAvailable)?;
    let manager = VolumeManager::new(Arc::new(docker));

    let details = manager.inspect(&name).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&details)?);
    } else {
        println!("{}", style("Volume Details").cyan().bold());
        println!("{}", "=".repeat(50));
        println!();
        println!("  Name:       {}", details.name);
        println!("  Driver:     {}", details.driver);
        println!("  Mountpoint: {}", details.mountpoint);
        if let Some(created) = &details.created_at {
            println!("  Created:    {}", created);
        }

        if !details.labels.is_empty() {
            println!();
            println!("  Labels:");
            for (key, value) in details.labels.iter() {
                println!("    {} = {}", key, value);
            }
        }

        if !details.options.is_empty() {
            println!();
            println!("  Options:");
            for (key, value) in details.options.iter() {
                println!("    {} = {}", key, value);
            }
        }
    }

    Ok(())
}

/// Truncate string to max length with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
