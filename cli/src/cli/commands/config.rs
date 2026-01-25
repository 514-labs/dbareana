use crate::config::{load_or_default, validate_config};
use crate::Result;
use console::style;
use std::path::PathBuf;

/// Handle `config validate` command
pub async fn handle_config_validate(
    config_path: Option<PathBuf>,
    check_scripts: bool,
) -> Result<()> {
    println!("{}", style("Validating configuration...").bold().cyan());

    // Load config
    let config = load_or_default(config_path.clone())?;

    // Validate config
    let validation_result = validate_config(&config)?;

    // Print warnings
    if !validation_result.warnings.is_empty() {
        println!("\n{}", style("Warnings:").yellow().bold());
        for warning in &validation_result.warnings {
            println!("  {} {}", style("⚠").yellow(), warning);
        }
    }

    // Check init scripts if requested
    if check_scripts {
        println!("\n{}", style("Checking initialization scripts...").cyan());

        let mut script_count = 0;
        let mut missing_count = 0;

        for (db_name, db_config) in &config.databases {
            for script in &db_config.init_scripts {
                script_count += 1;
                let path = PathBuf::from(script.path());

                if !path.exists() {
                    println!(
                        "  {} {} - {}",
                        style("✗").red(),
                        db_name,
                        style(script.path()).dim()
                    );
                    missing_count += 1;
                } else {
                    println!(
                        "  {} {} - {}",
                        style("✓").green(),
                        db_name,
                        style(script.path()).dim()
                    );
                }
            }
        }

        if script_count == 0 {
            println!("  {}", style("No initialization scripts configured").dim());
        } else if missing_count > 0 {
            return Err(crate::DBArenaError::ConfigError(format!(
                "{} initialization script(s) not found",
                missing_count
            )));
        }
    }

    println!(
        "\n{} Configuration is valid",
        style("✓").green().bold()
    );

    if let Some(path) = config_path {
        println!("  Config file: {}", path.display());
    } else {
        println!("  Using default configuration");
    }

    Ok(())
}

/// Handle `config show` command
pub async fn handle_config_show(
    config_path: Option<PathBuf>,
    profile: Option<String>,
) -> Result<()> {
    println!("{}", style("Configuration").bold().cyan());
    println!("{}", "─".repeat(60));

    // Load config
    let config = load_or_default(config_path.clone())?;

    // Show source
    if let Some(path) = config_path {
        println!("Source: {}", style(path.display()).dim());
    } else {
        println!("Source: {}", style("defaults").dim());
    }
    println!();

    // Show defaults
    if config.defaults.persistent.is_some()
        || config.defaults.memory_mb.is_some()
        || config.defaults.cpu_shares.is_some()
    {
        println!("{}", style("Defaults:").bold());
        if let Some(persistent) = config.defaults.persistent {
            println!("  persistent: {}", persistent);
        }
        if let Some(memory) = config.defaults.memory_mb {
            println!("  memory_mb: {}", memory);
        }
        if let Some(cpu) = config.defaults.cpu_shares {
            println!("  cpu_shares: {}", cpu);
        }
        println!();
    }

    // Show profiles
    if !config.profiles.is_empty() {
        println!("{}", style("Global Profiles:").bold());
        for (name, profile_config) in &config.profiles {
            println!("  {}:", style(name).cyan());
            if !profile_config.env.is_empty() {
                for (key, value) in &profile_config.env {
                    println!("    {} = {}", key, value);
                }
            }
        }
        println!();
    }

    // Show databases
    if !config.databases.is_empty() {
        println!("{}", style("Databases:").bold());
        for (db_name, db_config) in &config.databases {
            println!("  {}:", style(db_name).cyan());

            if let Some(version) = &db_config.default_version {
                println!("    default_version: {}", version);
            }

            if !db_config.env.is_empty() {
                println!("    env:");
                for (key, value) in &db_config.env {
                    println!("      {} = {}", key, value);
                }
            }

            if !db_config.profiles.is_empty() {
                println!("    profiles:");
                for (prof_name, prof_config) in &db_config.profiles {
                    println!("      {}:", style(prof_name).dim());
                    for (key, value) in &prof_config.env {
                        println!("        {} = {}", key, value);
                    }
                }
            }

            if !db_config.init_scripts.is_empty() {
                println!("    init_scripts:");
                for script in &db_config.init_scripts {
                    println!("      - {}", script.path());
                }
            }
        }
        println!();
    }

    // Show resolved profile if specified
    if let Some(profile_name) = profile {
        println!("{}", style(format!("Resolved Profile: {}", profile_name)).bold());

        for (db_name, _) in &config.databases {
            let db_type = crate::container::DatabaseType::from_string(db_name)
                .ok_or_else(|| crate::DBArenaError::ConfigError(format!("Unknown database type: {}", db_name)))?;

            match crate::config::resolve_profile(&config, &profile_name, db_type) {
                Ok(env_vars) => {
                    println!("  {} ({})", style(db_name).cyan(), profile_name);
                    if env_vars.is_empty() {
                        println!("    {}", style("(no environment variables)").dim());
                    } else {
                        for (key, value) in &env_vars {
                            println!("    {} = {}", key, value);
                        }
                    }
                }
                Err(e) => {
                    println!("  {} ({}): {}", style(db_name).cyan(), profile_name, style(e).red());
                }
            }
        }
    }

    Ok(())
}

/// Handle `config init` command
pub async fn handle_config_init() -> Result<()> {
    println!("{}", style("Initialize dbarena configuration").bold().cyan());
    println!("{}", "─".repeat(60));

    // Check if config already exists
    let config_path = PathBuf::from("./dbarena.toml");
    if config_path.exists() {
        let overwrite = dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt(format!(
                "{} already exists. Overwrite?",
                config_path.display()
            ))
            .default(false)
            .interact()
            .map_err(|e| crate::DBArenaError::Other(format!("Prompt failed: {}", e)))?;

        if !overwrite {
            println!("{}", style("Cancelled").yellow());
            return Ok(());
        }
    }

    // Create example configuration
    let example_config = r#"# dbarena Configuration File
# See https://github.com/yourusername/dbarena for documentation

# Global defaults
[defaults]
persistent = false
memory_mb = 512

# Environment profiles
[profiles.dev]
env = { LOG_LEVEL = "debug", ENVIRONMENT = "development" }

[profiles.prod]
env = { LOG_LEVEL = "error", ENVIRONMENT = "production" }

# PostgreSQL configuration
[databases.postgres]
default_version = "16"

[databases.postgres.env]
POSTGRES_USER = "postgres"
POSTGRES_PASSWORD = "postgres"
POSTGRES_DB = "myapp"

[databases.postgres.profiles.dev]
env = { POSTGRES_DB = "myapp_dev" }

[databases.postgres.profiles.prod]
env = { POSTGRES_DB = "myapp_prod", POSTGRES_PASSWORD = "CHANGE_ME" }

# Initialization scripts (uncomment to use)
# [databases.postgres]
# init_scripts = ["./schema.sql", "./seed.sql"]
"#;

    std::fs::write(&config_path, example_config)?;

    println!(
        "\n{} Configuration file created: {}",
        style("✓").green().bold(),
        style(config_path.display()).cyan()
    );

    println!("\n{}", style("Next steps:").bold());
    println!("  1. Edit {} to customize your configuration", config_path.display());
    println!("  2. Run 'dbarena config validate' to check your configuration");
    println!("  3. Run 'dbarena create postgres --profile dev' to use the configuration");

    Ok(())
}
