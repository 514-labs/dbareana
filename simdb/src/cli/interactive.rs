use crate::container::DatabaseType;
use crate::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};

#[derive(Debug)]
pub struct DatabaseSelection {
    pub database: DatabaseType,
    pub version: String,
}

/// Interactive prompt to select databases and their versions
pub fn select_databases() -> Result<Vec<DatabaseSelection>> {
    println!("\n{}", style("Interactive Database Selection").bold().cyan());
    println!("{}", "─".repeat(50));

    // Step 1: Select which databases to create
    let databases = vec!["PostgreSQL", "MySQL", "SQL Server"];

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select databases to create (use Space to select, Enter to confirm)")
        .items(&databases)
        .interact()
        .map_err(|e| crate::SimDbError::Other(format!("Selection failed: {}", e)))?;

    if selections.is_empty() {
        return Err(crate::SimDbError::InvalidConfig(
            "No databases selected".to_string(),
        ));
    }

    println!();

    // Step 2: For each selected database, choose a version
    let mut results = Vec::new();

    for &idx in &selections {
        let db_name = databases[idx];
        let (db_type, versions) = match idx {
            0 => {
                // PostgreSQL
                (
                    DatabaseType::Postgres,
                    vec![
                        "16 (latest)",
                        "15",
                        "14",
                        "13",
                        "12",
                        "11",
                        "Custom version",
                    ],
                )
            }
            1 => {
                // MySQL
                (
                    DatabaseType::MySQL,
                    vec![
                        "8.0 (latest)",
                        "8.4",
                        "5.7",
                        "5.6",
                        "Custom version",
                    ],
                )
            }
            2 => {
                // SQL Server
                (
                    DatabaseType::SQLServer,
                    vec![
                        "2022-latest",
                        "2019-latest",
                        "2017-latest",
                        "Custom version",
                    ],
                )
            }
            _ => unreachable!(),
        };

        println!("{}:", style(db_name).bold().green());

        let version_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("  Select version")
            .items(&versions)
            .default(0)
            .interact()
            .map_err(|e| crate::SimDbError::Other(format!("Selection failed: {}", e)))?;

        let version = if versions[version_idx].contains("Custom") {
            // Prompt for custom version
            let custom: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
                .with_prompt("  Enter custom version")
                .interact_text()
                .map_err(|e| crate::SimDbError::Other(format!("Input failed: {}", e)))?;
            custom
        } else {
            // Extract version number from selection (remove " (latest)" suffix)
            versions[version_idx]
                .split_whitespace()
                .next()
                .unwrap()
                .to_string()
        };

        println!("  {} {} {}", style("✓").green(), db_name, style(&version).dim());

        results.push(DatabaseSelection {
            database: db_type,
            version,
        });
    }

    println!();

    // Step 3: Confirm selections
    let summary: Vec<String> = results
        .iter()
        .map(|s| format!("{} ({})", s.database.as_str(), s.version))
        .collect();

    println!("{}", style("Selected databases:").bold());
    for item in &summary {
        println!("  • {}", style(item).cyan());
    }
    println!();

    let confirm = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Proceed with these selections?")
        .default(true)
        .interact()
        .map_err(|e| crate::SimDbError::Other(format!("Confirmation failed: {}", e)))?;

    if !confirm {
        return Err(crate::SimDbError::Other("Cancelled by user".to_string()));
    }

    Ok(results)
}

/// Interactive prompt for advanced options
pub fn prompt_advanced_options() -> Result<AdvancedOptions> {
    let configure_advanced = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Configure advanced options?")
        .default(false)
        .interact()
        .map_err(|e| crate::SimDbError::Other(format!("Prompt failed: {}", e)))?;

    if !configure_advanced {
        return Ok(AdvancedOptions::default());
    }

    println!();
    println!("{}", style("Advanced Configuration").bold().cyan());
    println!("{}", "─".repeat(50));

    // Memory limit
    let memory_input: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Memory limit in MB (leave empty for no limit)")
        .allow_empty(true)
        .interact_text()
        .map_err(|e| crate::SimDbError::Other(format!("Input failed: {}", e)))?;

    let memory = if memory_input.is_empty() {
        None
    } else {
        Some(
            memory_input
                .parse::<u64>()
                .map_err(|_| crate::SimDbError::InvalidConfig("Invalid memory value".to_string()))?,
        )
    };

    // CPU shares
    let cpu_input: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
        .with_prompt("CPU shares (leave empty for no limit)")
        .allow_empty(true)
        .interact_text()
        .map_err(|e| crate::SimDbError::Other(format!("Input failed: {}", e)))?;

    let cpu_shares = if cpu_input.is_empty() {
        None
    } else {
        Some(
            cpu_input
                .parse::<u64>()
                .map_err(|_| crate::SimDbError::InvalidConfig("Invalid CPU shares value".to_string()))?,
        )
    };

    // Persistent volume
    let persistent = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Use persistent volume?")
        .default(false)
        .interact()
        .map_err(|e| crate::SimDbError::Other(format!("Prompt failed: {}", e)))?;

    println!();

    Ok(AdvancedOptions {
        memory,
        cpu_shares,
        persistent,
    })
}

#[derive(Debug, Default)]
pub struct AdvancedOptions {
    pub memory: Option<u64>,
    pub cpu_shares: Option<u64>,
    pub persistent: bool,
}

/// Interactive container selection
pub fn select_container(containers: Vec<crate::container::Container>, action: &str) -> Result<String> {
    if containers.is_empty() {
        return Err(crate::SimDbError::Other(format!(
            "No containers available to {}",
            action
        )));
    }

    println!("\n{}", style(format!("Select container to {}", action)).bold().cyan());
    println!("{}", "─".repeat(50));

    let items: Vec<String> = containers
        .iter()
        .map(|c| {
            format!(
                "{:<20} {:<15} {:<12}",
                c.name,
                c.database_type,
                c.status.to_string()
            )
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Container")
        .items(&items)
        .interact()
        .map_err(|e| crate::SimDbError::Other(format!("Selection failed: {}", e)))?;

    Ok(containers[selection].name.clone())
}

/// Interactive multi-container selection
pub fn select_containers(
    containers: Vec<crate::container::Container>,
    action: &str,
) -> Result<Vec<String>> {
    if containers.is_empty() {
        return Err(crate::SimDbError::Other(format!(
            "No containers available to {}",
            action
        )));
    }

    println!("\n{}", style(format!("Select containers to {}", action)).bold().cyan());
    println!("{}", "─".repeat(50));

    let items: Vec<String> = containers
        .iter()
        .map(|c| {
            format!(
                "{:<20} {:<15} {:<12}",
                c.name,
                c.database_type,
                c.status.to_string()
            )
        })
        .collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Containers (use Space to select, Enter to confirm)")
        .items(&items)
        .interact()
        .map_err(|e| crate::SimDbError::Other(format!("Selection failed: {}", e)))?;

    if selections.is_empty() {
        return Err(crate::SimDbError::Other("No containers selected".to_string()));
    }

    Ok(selections
        .into_iter()
        .map(|idx| containers[idx].name.clone())
        .collect())
}
