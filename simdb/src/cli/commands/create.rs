use crate::cli::interactive;
use crate::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
use crate::health::{
    wait_for_healthy, MySQLHealthChecker, PostgresHealthChecker, SQLServerHealthChecker,
};
use crate::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
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

    let manager = ContainerManager::new(docker_client);

    // Create containers
    for selection in selections {
        let db_name = selection.database.as_str();
        if let Err(e) = create_single_database(
            &manager,
            selection.database,
            selection.version,
            name.clone(),
            port,
            persistent,
            memory,
            cpu_shares,
        )
        .await
        {
            error!("Failed to create {}: {}", db_name, e);
            eprintln!("{} Failed to create {}: {}", style("✗").red(), db_name, e);
        }
    }

    Ok(())
}

async fn create_single_database(
    manager: &ContainerManager,
    database: DatabaseType,
    version: String,
    name: Option<String>,
    port: Option<u16>,
    persistent: bool,
    memory: Option<u64>,
    cpu_shares: Option<u64>,
) -> Result<()> {
    println!(
        "\n{} Creating {} container...",
        style("→").cyan(),
        style(database.as_str()).bold()
    );

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
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    let image = database.docker_image(&config.version);
    pb.set_message(format!("Checking image {}...", image));

    let docker = DockerClient::new()?;
    if !docker.image_exists(&image).await? {
        pb.set_message(format!("Pulling image {}...", image));
        docker.pull_image(&image).await?;
        pb.finish_with_message(format!("✓ Pulled image {}", image));
    } else {
        pb.finish_with_message(format!("✓ Image {} already cached", image));
    }

    // Step 2: Create container
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Creating container...");

    let container = manager.create_container(config.clone()).await?;
    pb.finish_with_message(format!("✓ Created container {}", container.name));

    // Step 3: Start container
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Starting container...");

    manager.start_container(&container.id).await?;
    pb.finish_with_message("✓ Container started");

    // Step 4: Wait for healthy
    let checker: Box<dyn crate::health::HealthChecker> = match database {
        DatabaseType::Postgres => Box::new(PostgresHealthChecker::new(docker.docker().clone())),
        DatabaseType::MySQL => Box::new(MySQLHealthChecker::new(docker.docker().clone())),
        DatabaseType::SQLServer => Box::new(SQLServerHealthChecker::new(docker.docker().clone())),
    };

    wait_for_healthy(&container.id, checker.as_ref(), DEFAULT_HEALTH_TIMEOUT).await?;

    // Step 5: Display connection info
    println!("\n{}", style("Container ready!").green().bold());
    println!("  Name:     {}", style(&container.name).cyan());
    println!("  Database: {}", style(database.as_str()).cyan());
    println!("  Version:  {}", style(&container.version).cyan());

    if let Some(host_port) = container.host_port {
        println!("  Port:     {}", style(host_port).cyan());

        // Display connection strings
        match database {
            DatabaseType::Postgres => {
                println!("\n{}", style("Connection:").bold());
                println!(
                    "  {}",
                    style(format!(
                        "psql -h localhost -p {} -U postgres -d testdb",
                        host_port
                    ))
                    .dim()
                );
                println!(
                    "  {}",
                    style(format!(
                        "postgres://postgres:postgres@localhost:{}/testdb",
                        host_port
                    ))
                    .dim()
                );
            }
            DatabaseType::MySQL => {
                println!("\n{}", style("Connection:").bold());
                println!(
                    "  {}",
                    style(format!(
                        "mysql -h localhost -P {} -u root -pmysql testdb",
                        host_port
                    ))
                    .dim()
                );
                println!(
                    "  {}",
                    style(format!(
                        "mysql://root:mysql@localhost:{}/testdb",
                        host_port
                    ))
                    .dim()
                );
            }
            DatabaseType::SQLServer => {
                println!("\n{}", style("Connection:").bold());
                println!(
                    "  {}",
                    style(format!(
                        "sqlcmd -S localhost,{} -U sa -P 'YourStrong@Passw0rd'",
                        host_port
                    ))
                    .dim()
                );
            }
        }
    }

    Ok(())
}
