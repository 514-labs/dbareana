use crate::cli::interactive;
use crate::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
use crate::health::{
    wait_for_healthy, MySQLHealthChecker, PostgresHealthChecker, SQLServerHealthChecker,
};
use crate::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info};

const DEFAULT_HEALTH_TIMEOUT: Duration = Duration::from_secs(60);

#[allow(clippy::too_many_arguments)]
pub async fn handle_create(
    databases: Vec<String>,
    interactive_mode: bool,
    version: Option<String>,
    name: Option<String>,
    port: Option<u16>,
    persistent: bool,
    memory: Option<u64>,
    cpu_shares: Option<u64>,
) -> Result<()> {
    info!("Starting create command");

    // Handle interactive mode
    let (selections, memory, cpu_shares, persistent) = if interactive_mode {
        let selections = interactive::select_databases()?;

        // Prompt for advanced options
        let advanced = interactive::prompt_advanced_options()?;
        let memory = advanced.memory;
        let cpu_shares = advanced.cpu_shares;
        let persistent = advanced.persistent;

        (selections, memory, cpu_shares, persistent)
    } else {
        // Validate that at least one database was specified
        if databases.is_empty() {
            return Err(crate::DBArenaError::InvalidConfig(
                "At least one database type must be specified. Use -i for interactive mode or provide database names.".to_string(),
            ));
        }

        // Convert CLI args to selections
        let selections = databases
            .into_iter()
            .map(|db_name| {
                let database = DatabaseType::from_string(&db_name).ok_or_else(|| {
                    crate::DBArenaError::InvalidConfig(format!(
                        "Unknown database type: {}",
                        db_name
                    ))
                })?;
                Ok(interactive::DatabaseSelection {
                    database,
                    version: version
                        .clone()
                        .unwrap_or_else(|| database.default_version().to_string()),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        (selections, memory, cpu_shares, persistent)
    };

    // Initialize Docker client
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Connecting to Docker...");

    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;
    pb.finish_with_message("✓ Connected to Docker");

    let manager = Arc::new(ContainerManager::new(docker_client));

    println!("\n{}", style("Creating containers...").bold().cyan());
    println!("{}", "─".repeat(80));

    let start_time = Instant::now();

    // Show initial status
    for selection in &selections {
        let display_name = format!("{}-{}", selection.database.as_str(), selection.version);
        println!(
            "  {} {} - Queued",
            style("○").dim(),
            style(&display_name).cyan()
        );
    }
    println!();

    // Spawn tasks for all containers
    let mut tasks = Vec::new();

    for selection in selections {
        let manager_clone = Arc::clone(&manager);
        let name_clone = name.clone();
        let db_label = format!("{} ({})", selection.database.as_str(), selection.version);
        let display_name = format!("{}-{}", selection.database.as_str(), selection.version);

        let task = tokio::spawn(async move {
            let result = create_single_database_simple(
                &manager_clone,
                selection.database,
                selection.version.clone(),
                name_clone,
                port,
                persistent,
                memory,
                cpu_shares,
            )
            .await;

            (display_name, result)
        });

        tasks.push((task, db_label));
    }

    // Wait for all container creation tasks to complete and show results as they finish
    let mut results = Vec::new();
    for (task, db_label) in tasks {
        match task.await {
            Ok((display_name, result)) => {
                match &result {
                    Ok(_) => println!(
                        "  {} {} - Ready",
                        style("✓").green(),
                        style(&display_name).cyan()
                    ),
                    Err(e) => println!(
                        "  {} {} - Failed: {}",
                        style("✗").red(),
                        style(&display_name).cyan(),
                        e
                    ),
                }
                results.push((db_label, result));
            }
            Err(e) => {
                error!("Task panicked for {}: {}", db_label, e);
                results.push((
                    db_label,
                    Err(crate::DBArenaError::Other(format!("Task failed: {}", e))),
                ));
            }
        }
    }

    let elapsed = start_time.elapsed();

    // Print summary
    println!("\n{}", "─".repeat(80));
    println!("{}", style("Summary").bold().cyan());
    println!("{}", "─".repeat(80));

    let mut success_count = 0;
    let mut failed_count = 0;

    for (db_name, result) in results {
        match result {
            Ok(_) => {
                println!(
                    "  {} {} - Created successfully",
                    style("✓").green(),
                    style(db_name).bold()
                );
                success_count += 1;
            }
            Err(e) => {
                println!(
                    "  {} {} - Failed: {}",
                    style("✗").red(),
                    style(db_name).bold(),
                    e
                );
                failed_count += 1;
            }
        }
    }

    println!(
        "\n{} in {:.2}s",
        style(format!(
            "{} successful, {} failed",
            success_count, failed_count
        ))
        .dim(),
        elapsed.as_secs_f64()
    );

    if failed_count > 0 {
        return Err(crate::DBArenaError::Other(format!(
            "{} container(s) failed to create",
            failed_count
        )));
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn create_single_database_simple(
    manager: &ContainerManager,
    database: DatabaseType,
    version: String,
    name: Option<String>,
    port: Option<u16>,
    persistent: bool,
    memory: Option<u64>,
    cpu_shares: Option<u64>,
) -> Result<()> {
    // Build configuration
    let mut config = ContainerConfig::new(database);
    config = config.with_version(version);
    if let Some(n) = name {
        config = config.with_name(n);
    }
    if let Some(p) = port {
        config = config.with_port(p);
    }
    config = config.with_persistent(persistent);
    if let Some(m) = memory {
        config = config.with_memory_limit(m);
    }
    if let Some(c) = cpu_shares {
        config = config.with_cpu_shares(c);
    }

    // Step 1: Ensure image is available
    let image = database.docker_image(&config.version);
    let docker = DockerClient::new()?;

    if !docker.image_exists(&image).await? {
        docker.pull_image(&image).await?;
    }

    // Step 2: Create container
    let container = manager.create_container(config.clone()).await?;

    // Step 3: Start container
    manager.start_container(&container.id).await?;

    // Step 4: Wait for healthy
    let checker: Box<dyn crate::health::HealthChecker> = match database {
        DatabaseType::Postgres => Box::new(PostgresHealthChecker::new(docker.docker().clone())),
        DatabaseType::MySQL => Box::new(MySQLHealthChecker::new(docker.docker().clone())),
        DatabaseType::SQLServer => Box::new(SQLServerHealthChecker::new(docker.docker().clone())),
    };

    wait_for_healthy(&container.id, checker.as_ref(), DEFAULT_HEALTH_TIMEOUT).await?;

    Ok(())
}
