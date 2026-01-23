use crate::cli::interactive;
use crate::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
use crate::health::{
    wait_for_healthy, MySQLHealthChecker, PostgresHealthChecker, SQLServerHealthChecker,
};
use crate::Result;
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info};

const DEFAULT_HEALTH_TIMEOUT: Duration = Duration::from_secs(60);

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
            return Err(crate::SimDbError::InvalidConfig(
                "At least one database type must be specified. Use -i for interactive mode or provide database names.".to_string(),
            ));
        }

        // Convert CLI args to selections
        let selections = databases
            .into_iter()
            .map(|db_name| {
                let database = DatabaseType::from_string(&db_name).ok_or_else(|| {
                    crate::SimDbError::InvalidConfig(format!("Unknown database type: {}", db_name))
                })?;
                Ok(interactive::DatabaseSelection {
                    database,
                    version: version.clone().unwrap_or_else(|| database.default_version().to_string()),
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

    // Create multi-progress for parallel operations
    let multi_progress = MultiProgress::new();

    println!("\n{}", style("Creating containers...").bold().cyan());
    println!("{}", "─".repeat(80));

    let start_time = Instant::now();

    // Create all containers in parallel
    let mut tasks = Vec::new();

    for selection in selections {
        let manager_clone = Arc::clone(&manager);
        let mp = multi_progress.clone();
        let name_clone = name.clone();

        let task = tokio::spawn(async move {
            create_single_database_with_progress(
                &manager_clone,
                selection.database,
                selection.version,
                name_clone,
                port,
                persistent,
                memory,
                cpu_shares,
                mp,
            )
            .await
        });

        tasks.push((task, selection.database.as_str()));
    }

    // Wait for all tasks to complete
    let mut results = Vec::new();
    for (task, db_name) in tasks {
        match task.await {
            Ok(result) => results.push((db_name, result)),
            Err(e) => {
                error!("Task panicked for {}: {}", db_name, e);
                results.push((db_name, Err(crate::SimDbError::Other(format!("Task failed: {}", e)))));
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
                println!("  {} {} - Created successfully", style("✓").green(), style(db_name).bold());
                success_count += 1;
            }
            Err(e) => {
                println!("  {} {} - Failed: {}", style("✗").red(), style(db_name).bold(), e);
                failed_count += 1;
            }
        }
    }

    println!("\n{} in {:.2}s",
        style(format!("{} successful, {} failed", success_count, failed_count)).dim(),
        elapsed.as_secs_f64()
    );

    if failed_count > 0 {
        return Err(crate::SimDbError::Other(format!("{} container(s) failed to create", failed_count)));
    }

    Ok(())
}

async fn create_single_database_with_progress(
    manager: &ContainerManager,
    database: DatabaseType,
    version: String,
    name: Option<String>,
    port: Option<u16>,
    persistent: bool,
    memory: Option<u64>,
    cpu_shares: Option<u64>,
    multi_progress: MultiProgress,
) -> Result<()> {
    // Create progress bar for this database
    let pb = multi_progress.add(ProgressBar::new(4));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix} {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓▒░ "),
    );
    pb.set_prefix(format!("{:12}", style(database.as_str()).bold().cyan()));

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
    pb.set_position(0);
    pb.set_message(format!("{} Checking image...", style("📦").cyan()));

    let image = database.docker_image(&config.version);
    let docker = DockerClient::new()?;

    if !docker.image_exists(&image).await? {
        pb.set_message(format!("{} Pulling image...", style("⬇").cyan()));
        docker.pull_image(&image).await?;
    }

    pb.inc(1);
    pb.set_message(format!("{} Image ready", style("✓").green()));

    // Step 2: Create container
    pb.set_message(format!("{} Creating container...", style("🔧").cyan()));
    let container = manager.create_container(config.clone()).await?;
    pb.inc(1);
    pb.set_message(format!("{} Container created", style("✓").green()));

    // Step 3: Start container
    pb.set_message(format!("{} Starting...", style("▶").cyan()));
    manager.start_container(&container.id).await?;
    pb.inc(1);
    pb.set_message(format!("{} Started", style("✓").green()));

    // Step 4: Wait for healthy
    pb.set_message(format!("{} Health check...", style("🏥").cyan()));
    let checker: Box<dyn crate::health::HealthChecker> = match database {
        DatabaseType::Postgres => Box::new(PostgresHealthChecker::new(docker.docker().clone())),
        DatabaseType::MySQL => Box::new(MySQLHealthChecker::new(docker.docker().clone())),
        DatabaseType::SQLServer => Box::new(SQLServerHealthChecker::new(docker.docker().clone())),
    };

    wait_for_healthy(&container.id, checker.as_ref(), DEFAULT_HEALTH_TIMEOUT).await?;
    pb.inc(1);
    pb.set_message(format!("{} Healthy", style("✓").green().bold()));
    pb.finish();

    Ok(())
}

