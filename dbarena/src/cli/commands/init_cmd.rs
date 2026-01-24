use crate::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
use crate::init::{execute_init_scripts, LogManager};
use crate::Result;
use console::style;
use std::path::PathBuf;

/// Handle `init test` command - test a script against a running container
pub async fn handle_init_test(script: PathBuf, container: String) -> Result<()> {
    println!(
        "{}",
        style(format!("Testing script: {}", script.display()))
            .bold()
            .cyan()
    );
    println!("Target container: {}", style(&container).cyan());
    println!("{}", "─".repeat(60));

    // Verify script exists
    if !script.exists() {
        return Err(crate::DBArenaError::InitScriptNotFound(format!(
            "Script not found: {}",
            script.display()
        )));
    }

    // Connect to Docker
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client);

    // Find container
    let container_info = manager
        .find_container(&container)
        .await?
        .ok_or_else(|| {
            crate::DBArenaError::ContainerNotFound(format!(
                "Container not found: {}",
                container
            ))
        })?;

    // Determine database type
    let db_type = DatabaseType::from_string(&container_info.database_type).ok_or_else(|| {
        crate::DBArenaError::Other(format!(
            "Unknown database type: {}",
            container_info.database_type
        ))
    })?;

    println!(
        "Database: {} ({})",
        style(&container_info.database_type).green(),
        container_info.version
    );
    println!();

    // Create a temporary container config (we need this for env vars)
    let config = ContainerConfig::new(db_type)
        .with_version(container_info.version.clone())
        .with_init_scripts(vec![script.clone()]);

    // Execute script
    let log_manager = LogManager::new(None)?;

    println!("{}", style("Executing script...").cyan());
    let start = std::time::Instant::now();

    // Create a new Docker client for script execution
    let docker_for_exec = DockerClient::new()?;

    let results = execute_init_scripts(
        docker_for_exec.docker(),
        &container_info.id,
        vec![script.clone()],
        db_type,
        &config,
        false, // Don't continue on error
        &log_manager,
    )
    .await?;

    let duration = start.elapsed();

    // Display results
    if let Some(result) = results.first() {
        if result.success {
            println!();
            println!("{} Script executed successfully", style("✓").green().bold());
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
            println!();
            println!("{} Script execution failed", style("✗").red().bold());
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
    let logs = log_manager.get_session_logs(&container_info.id)?;
    if let Some(log_entry) = logs.first() {
        println!();
        println!(
            "Log saved to: {}",
            style(log_entry.log_path.display()).dim()
        );
    }

    Ok(())
}

/// Handle `init validate` command - basic validation of SQL script
pub async fn handle_init_validate(script: PathBuf, database: String) -> Result<()> {
    println!(
        "{}",
        style(format!("Validating script: {}", script.display()))
            .bold()
            .cyan()
    );
    println!("Database type: {}", style(&database).cyan());
    println!("{}", "─".repeat(60));

    // Verify script exists
    if !script.exists() {
        return Err(crate::DBArenaError::InitScriptNotFound(format!(
            "Script not found: {}",
            script.display()
        )));
    }

    // Parse database type
    let _db_type = DatabaseType::from_string(&database).ok_or_else(|| {
        crate::DBArenaError::InvalidConfig(format!("Unknown database type: {}", database))
    })?;

    // Read script content
    let content = std::fs::read_to_string(&script)?;

    // Basic validation
    let mut issues = Vec::new();

    // Check if file is empty
    if content.trim().is_empty() {
        issues.push("Script is empty");
    }

    // Check for common SQL syntax patterns
    let has_sql = content.to_uppercase().contains("SELECT")
        || content.to_uppercase().contains("INSERT")
        || content.to_uppercase().contains("UPDATE")
        || content.to_uppercase().contains("DELETE")
        || content.to_uppercase().contains("CREATE")
        || content.to_uppercase().contains("ALTER")
        || content.to_uppercase().contains("DROP");

    if !has_sql {
        issues.push("Script doesn't appear to contain SQL statements");
    }

    // Check for common typos
    if content.contains("INSRT ") {
        issues.push("Potential typo: 'INSRT' (should be 'INSERT'?)");
    }
    if content.contains("SLECT ") {
        issues.push("Potential typo: 'SLECT' (should be 'SELECT'?)");
    }
    if content.contains("UPDTE ") {
        issues.push("Potential typo: 'UPDTE' (should be 'UPDATE'?)");
    }

    // Display results
    println!();
    if issues.is_empty() {
        println!(
            "{} Script appears valid",
            style("✓").green().bold()
        );
        println!("  Lines: {}", content.lines().count());
        println!("  Size: {} bytes", content.len());
    } else {
        println!(
            "{} Issues found:",
            style("⚠").yellow().bold()
        );
        for issue in &issues {
            println!("  • {}", issue);
        }
    }

    println!();
    println!(
        "{}",
        style("Note: This is basic validation only.").dim()
    );
    println!(
        "{}",
        style("Use 'dbarena init test' to test against a real database.")
            .dim()
    );

    Ok(())
}
