use std::sync::Arc;
use bollard::Docker;
use console::style;

use crate::error::{DBArenaError, Result};
use crate::snapshot::SnapshotManager;

/// Handle snapshot create command
pub async fn handle_snapshot_create(
    container: String,
    name: String,
    message: Option<String>,
) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|_| DBArenaError::DockerNotAvailable)?;
    let manager = SnapshotManager::new(Arc::new(docker));

    println!(
        "{} Creating snapshot of container {}...",
        style("→").cyan(),
        style(&container).bold()
    );

    // Create the snapshot (auto_pause = true by default)
    let snapshot = manager.create(&container, name, message, true).await?;

    println!("  {} Snapshot created successfully", style("✓").green());
    println!();
    println!("  ID:       {}", snapshot.id);
    println!("  Name:     {}", snapshot.name);
    println!("  Image:    {}", snapshot.image_tag);
    println!("  Database: {}", snapshot.database_type);
    if let Some(msg) = &snapshot.message {
        println!("  Message:  {}", msg);
    }
    println!(
        "  Created:  {}",
        chrono::DateTime::from_timestamp(snapshot.created_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    );

    Ok(())
}

/// Handle snapshot list command
pub async fn handle_snapshot_list(json: bool) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|_| DBArenaError::DockerNotAvailable)?;
    let manager = SnapshotManager::new(Arc::new(docker));

    let snapshots = manager.list().await?;

    if snapshots.is_empty() {
        if !json {
            println!("{}", style("No snapshots found.").yellow());
        } else {
            println!("[]");
        }
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&snapshots)?);
    } else {
        println!(
            "{:<20} {:<30} {:<15} {:<20}",
            "NAME", "ID", "DATABASE", "CREATED"
        );
        println!("{}", "─".repeat(85));

        for snapshot in snapshots {
            let created = chrono::DateTime::from_timestamp(snapshot.created_at, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            println!(
                "{:<20} {:<30} {:<15} {:<20}",
                truncate_string(&snapshot.name, 20),
                truncate_string(&snapshot.id, 30),
                snapshot.database_type,
                created
            );
        }
    }

    Ok(())
}

/// Handle snapshot restore command
pub async fn handle_snapshot_restore(
    snapshot: String,
    name: Option<String>,
    port: Option<u16>,
) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|_| DBArenaError::DockerNotAvailable)?;
    let manager = SnapshotManager::new(Arc::new(docker));

    println!(
        "{} Restoring snapshot {}...",
        style("→").cyan(),
        style(&snapshot).bold()
    );

    let container = manager.restore(&snapshot, name.clone(), port).await?;

    println!("  {} Container restored successfully", style("✓").green());
    println!();
    println!("  Container ID:   {}", container.id);
    println!("  Name:           {}", container.name);
    println!("  Database:       {}", container.database_type);
    println!("  Port:           {}", container.port);
    if let Some(host_port) = container.host_port {
        println!("  Host Port:      {}", host_port);
    }
    println!();
    println!(
        "{}",
        style("Container is starting from snapshot!").green().bold()
    );

    Ok(())
}

/// Handle snapshot delete command
pub async fn handle_snapshot_delete(snapshot: String, yes: bool) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|_| DBArenaError::DockerNotAvailable)?;
    let manager = SnapshotManager::new(Arc::new(docker));

    // Get snapshot info first
    let snap_info = manager.get(&snapshot).await?;

    if !yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!(
                "Delete snapshot '{}' ({})? This cannot be undone.",
                snap_info.name, snap_info.id
            ))
            .interact()
            .map_err(|e| DBArenaError::Other(format!("Failed to read input: {}", e)))?;

        if !confirmed {
            println!("{}", style("Cancelled.").yellow());
            return Ok(());
        }
    }

    println!(
        "{} Deleting snapshot {}...",
        style("→").cyan(),
        style(&snapshot).bold()
    );

    manager.delete(&snapshot).await?;

    println!("  {} Snapshot deleted successfully", style("✓").green());

    Ok(())
}

/// Handle snapshot inspect command
pub async fn handle_snapshot_inspect(snapshot: String, json: bool) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()
        .map_err(|_| DBArenaError::DockerNotAvailable)?;
    let manager = SnapshotManager::new(Arc::new(docker));

    let snap = manager.inspect(&snapshot).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&snap)?);
    } else {
        println!("{}", style("Snapshot Details").cyan().bold());
        println!("{}", "=".repeat(50));
        println!();
        println!("  ID:               {}", snap.id);
        println!("  Name:             {}", snap.name);
        println!("  Source Container: {}", snap.source_container);
        println!("  Database Type:    {}", snap.database_type);
        println!("  Image Tag:        {}", snap.image_tag);
        println!(
            "  Created:          {}",
            chrono::DateTime::from_timestamp(snap.created_at, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        );
        if let Some(msg) = &snap.message {
            println!("  Message:          {}", msg);
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
