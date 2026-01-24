use crate::cli::interactive;
use crate::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
use crate::init::{execute_init_scripts, LogManager};
use crate::Result;
use console::style;
use std::path::PathBuf;

/// Handle `exec` command - execute SQL on a running container
pub async fn handle_exec(
    container: Option<String>,
    interactive_mode: bool,
    script: Option<String>,
    file: Option<PathBuf>,
) -> Result<()> {
    // Validate that either script or file is provided
    if script.is_none() && file.is_none() {
        return Err(crate::DBArenaError::InvalidConfig(
            "Either --script or --file must be provided".to_string(),
        ));
    }

    if script.is_some() && file.is_some() {
        return Err(crate::DBArenaError::InvalidConfig(
            "Cannot specify both --script and --file".to_string(),
        ));
    }

    // Connect to Docker
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);

    // Get container (interactive or specified)
    let container_name = if interactive_mode || container.is_none() {
        // List all running containers
        let containers = manager.list_containers(false).await?;

        if containers.is_empty() {
            return Err(crate::DBArenaError::Other(
                "No running containers found".to_string(),
            ));
        }

        interactive::select_container(containers, "execute SQL on")?
    } else {
        container.unwrap()
    };

    // Find container info
    let container_info = manager
        .find_container(&container_name)
        .await?
        .ok_or_else(|| {
            crate::DBArenaError::ContainerNotFound(format!(
                "Container not found: {}",
                container_name
            ))
        })?;

    // Create a new Docker client for script execution
    let docker_for_exec = DockerClient::new()?;

    // Determine database type
    let db_type = DatabaseType::from_string(&container_info.database_type).ok_or_else(|| {
        crate::DBArenaError::Other(format!(
            "Unknown database type: {}",
            container_info.database_type
        ))
    })?;

    println!(
        "\n{}",
        style("Execute SQL on Container").bold().cyan()
    );
    println!("{}", "─".repeat(60));
    println!("Container: {}", style(&container_info.name).cyan());
    println!(
        "Database: {} ({})",
        style(&container_info.database_type).green(),
        container_info.version
    );
    println!();

    // Prepare script
    let (script_path, is_temp) = if let Some(inline_script) = script {
        // Create temporary file for inline script
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("dbarena-exec-{}.sql", uuid::Uuid::new_v4()));

        std::fs::write(&temp_file, inline_script)?;

        println!("{}", style("Executing inline SQL...").cyan());
        println!();
        (temp_file, true)
    } else if let Some(file_path) = file {
        // Use provided file
        if !file_path.exists() {
            return Err(crate::DBArenaError::InitScriptNotFound(format!(
                "Script file not found: {}",
                file_path.display()
            )));
        }

        println!(
            "{}",
            style(format!("Executing script: {}", file_path.display())).cyan()
        );
        println!();
        (file_path, false)
    } else {
        unreachable!()
    };

    // Create a temporary container config (we need this for env vars)
    let config = ContainerConfig::new(db_type)
        .with_version(container_info.version.clone())
        .with_init_scripts(vec![script_path.clone()]);

    // Execute script
    let log_manager = LogManager::new(None)?;

    let start = std::time::Instant::now();

    let results = execute_init_scripts(
        docker_for_exec.docker(),
        &container_info.id,
        vec![script_path.clone()],
        db_type,
        &config,
        false, // Don't continue on error
        &log_manager,
    )
    .await;

    // Clean up temp file if it was created
    if is_temp {
        let _ = std::fs::remove_file(&script_path);
    }

    let results = results?;
    let duration = start.elapsed();

    // Display results
    if let Some(result) = results.first() {
        if result.success {
            println!(
                "{} Script executed successfully",
                style("✓").green().bold()
            );
            println!("  Duration: {:.2}s", duration.as_secs_f64());
            println!("  Statements: {}", result.statements_executed);

            if !result.output.is_empty() {
                println!();
                println!("{}", style("Output:").bold());
                println!("{}", "─".repeat(60));
                println!("{}", result.output);
                println!("{}", "─".repeat(60));
            }
        } else {
            println!(
                "{} Script execution failed",
                style("✗").red().bold()
            );
            println!("  Duration: {:.2}s", duration.as_secs_f64());

            if let Some(error) = &result.error {
                println!();
                println!("{}", style("Error Details:").red().bold());
                println!("{}", "─".repeat(60));
                println!("{}", error);
                println!("{}", "─".repeat(60));
            }

            return Err(crate::DBArenaError::InitScriptFailed(
                "Script execution failed".to_string(),
            ));
        }
    }

    // Show log file location
    if !is_temp {
        let logs = log_manager.get_session_logs(&container_info.id)?;
        if let Some(log_entry) = logs.first() {
            println!();
            println!(
                "Log saved to: {}",
                style(log_entry.log_path.display()).dim()
            );
        }
    }

    Ok(())
}
