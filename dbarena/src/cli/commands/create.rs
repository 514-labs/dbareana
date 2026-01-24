use crate::cli::interactive;
use crate::config::{load_or_default, resolve_profile, get_database_env, merge_env_vars};
use crate::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
use crate::health::{
    wait_for_healthy, MySQLHealthChecker, PostgresHealthChecker, SQLServerHealthChecker,
};
use crate::init::{execute_init_scripts, LogManager};
use crate::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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
    config_path: Option<PathBuf>,
    profile: Option<String>,
    env_args: Vec<String>,
    env_file: Option<PathBuf>,
    init_scripts: Vec<PathBuf>,
    continue_on_error: bool,
    _keep_on_error: bool,
    _log_dir: Option<PathBuf>,
    _script_timeout: u64,
    _validate_only: bool,
) -> Result<()> {
    info!("Starting create command");

    // Load configuration file
    let config = load_or_default(config_path)?;

    // Helper function to parse KEY=VALUE env args
    let parse_env_args = |args: &[String]| -> Result<HashMap<String, String>> {
        let mut env_map = HashMap::new();
        for arg in args {
            let parts: Vec<&str> = arg.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(crate::DBArenaError::InvalidEnvVar(format!(
                    "Invalid environment variable format: '{}'. Expected KEY=VALUE",
                    arg
                )));
            }
            env_map.insert(parts[0].to_string(), parts[1].to_string());
        }
        Ok(env_map)
    };

    // Helper function to parse env file
    let parse_env_file = |path: PathBuf| -> Result<HashMap<String, String>> {
        let content = fs::read_to_string(&path).map_err(|e| {
            crate::DBArenaError::ConfigError(format!(
                "Failed to read env file '{}': {}",
                path.display(),
                e
            ))
        })?;

        let mut env_map = HashMap::new();
        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(crate::DBArenaError::InvalidEnvVar(format!(
                    "Invalid format in env file '{}' at line {}: '{}'. Expected KEY=VALUE",
                    path.display(),
                    line_num + 1,
                    line
                )));
            }
            env_map.insert(parts[0].to_string(), parts[1].to_string());
        }
        Ok(env_map)
    };

    // Parse CLI env args
    let cli_env = parse_env_args(&env_args)?;

    // Load env file if specified
    let file_env = if let Some(env_file_path) = env_file {
        parse_env_file(env_file_path)?
    } else {
        HashMap::new()
    };

    // Handle interactive mode
    let (selections, memory, cpu_shares, persistent, interactive_profile) = if interactive_mode {
        let selections = interactive::select_databases()?;

        // Prompt for profile selection if config has profiles
        let interactive_profile = if !config.profiles.is_empty() ||
            selections.iter().any(|s| {
                let db_key = s.database.to_string().to_lowercase();
                config.databases.get(&db_key).map(|d| !d.profiles.is_empty()).unwrap_or(false)
            }) {
            // Use first selected database for profile prompt (profiles will apply to all)
            interactive::select_profile(&config, selections[0].database)?
        } else {
            None
        };

        // Prompt for advanced options
        let advanced = interactive::prompt_advanced_options()?;
        let memory = advanced.memory;
        let cpu_shares = advanced.cpu_shares;
        let persistent = advanced.persistent;

        (selections, memory, cpu_shares, persistent, interactive_profile)
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
        (selections, memory, cpu_shares, persistent, None)
    };

    // Use interactive profile or CLI profile
    let profile = interactive_profile.or(profile);

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

        let config_clone = config.clone();
        let profile_clone = profile.clone();
        let cli_env_clone = cli_env.clone();
        let file_env_clone = file_env.clone();
        let init_scripts_clone = init_scripts.clone();

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
                config_clone,
                profile_clone,
                cli_env_clone,
                file_env_clone,
                init_scripts_clone,
                continue_on_error,
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
    db_config: crate::config::DBArenaConfig,
    profile: Option<String>,
    cli_env: HashMap<String, String>,
    file_env: HashMap<String, String>,
    init_scripts: Vec<PathBuf>,
    continue_on_error: bool,
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

    // Build environment variables with proper precedence:
    // 1. Database base env vars from config
    // 2. Profile env vars (if profile specified)
    // 3. Env file env vars
    // 4. CLI env vars (highest precedence)
    let mut layers = vec![get_database_env(&db_config, database)];

    if let Some(profile_name) = profile {
        let profile_env = resolve_profile(&db_config, &profile_name, database)?;
        layers.push(profile_env);
    }

    layers.push(file_env);
    layers.push(cli_env);

    let env_vars = merge_env_vars(layers);

    // Apply env vars and init scripts to config
    config = config.with_env_vars(env_vars);
    config = config.with_init_scripts(init_scripts);
    config = config.with_continue_on_error(continue_on_error);

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

    // Step 5: Execute initialization scripts (if any)
    if !config.init_scripts.is_empty() {
        let log_manager = LogManager::new(None)?;
        let results = execute_init_scripts(
            docker.docker(),
            &container.id,
            config.init_scripts.clone(),
            database,
            &config,
            config.continue_on_error,
            &log_manager,
        )
        .await?;

        // Check if any scripts failed
        let failed_scripts: Vec<_> = results.iter().filter(|r| !r.success).collect();
        if !failed_scripts.is_empty() && !config.continue_on_error {
            // Format error message with details
            let error_details: Vec<String> = failed_scripts
                .iter()
                .map(|r| {
                    format!(
                        "  - {}: {}",
                        r.script_path.display(),
                        r.error
                            .as_ref()
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| "Unknown error".to_string())
                    )
                })
                .collect();

            return Err(crate::DBArenaError::InitScriptFailed(format!(
                "Initialization scripts failed:\n{}",
                error_details.join("\n")
            )));
        }
    }

    Ok(())
}
