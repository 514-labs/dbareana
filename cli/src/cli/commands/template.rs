use crate::config::{Template, TemplateManager};
use crate::container::{ContainerConfig, ContainerManager, DockerClient};
use crate::{DBArenaError, Result};
use console::style;
use std::path::PathBuf;

pub async fn handle_template_save(
    container: String,
    name: String,
    description: Option<String>,
) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client.clone());

    // Find the container
    let found = manager
        .find_container(&container)
        .await?
        .ok_or_else(|| DBArenaError::ContainerNotFound(container.clone()))?;

    println!(
        "{} Saving container {} as template {}...",
        style("→").cyan(),
        style(&found.name).bold(),
        style(&name).bold()
    );

    // Create a basic container config from the found container
    // TODO: Extract full config from running container
    let config = ContainerConfig {
        name: Some(found.name.clone()),
        database: crate::container::DatabaseType::from_string(&found.database_type)
            .ok_or_else(|| {
                DBArenaError::InvalidConfig(format!(
                    "Unknown database type: {}",
                    found.database_type
                ))
            })?,
        version: found.version.clone(),
        port: Some(found.port),
        persistent: false,
        memory_limit: None,
        cpu_shares: None,
        env_vars: Default::default(),
        init_scripts: Vec::new(),
        continue_on_error: false,
        volumes: Vec::new(),
    };

    let template = Template::from_container_config(name.clone(), description.clone(), &config);

    let template_manager = TemplateManager::new()?;
    template_manager.save(&template)?;

    println!("  {} Template saved successfully\n", style("✓").green());
    println!("  Name:        {}", template.name);
    println!("  Database:    {}", template.database);
    if let Some(desc) = &template.description {
        println!("  Description: {}", desc);
    }

    Ok(())
}

pub async fn handle_template_list(json: bool) -> Result<()> {
    let template_manager = TemplateManager::new()?;
    let templates = template_manager.list()?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&templates)
                .map_err(|e| DBArenaError::Other(format!("JSON serialization failed: {}", e)))?
        );
        return Ok(());
    }

    if templates.is_empty() {
        println!("No templates found.");
        println!("\nCreate a template with:");
        println!("  dbarena template save <container> --name <template-name>");
        return Ok(());
    }

    // Print header
    println!(
        "{:<25} {:<15} {:<40}",
        "NAME", "DATABASE", "DESCRIPTION"
    );
    println!("{}", "─".repeat(80));

    // Print templates
    for template in templates {
        println!(
            "{:<25} {:<15} {:<40}",
            template.name,
            template.database,
            template
                .description
                .unwrap_or_else(|| "-".to_string())
                .chars()
                .take(40)
                .collect::<String>()
        );
    }

    Ok(())
}

pub async fn handle_template_delete(name: String, yes: bool) -> Result<()> {
    let template_manager = TemplateManager::new()?;

    // Verify template exists
    let _ = template_manager.load(&name)?;

    // Confirm deletion unless --yes flag is used
    if !yes {
        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt(format!("Delete template '{}'?", name))
            .default(false)
            .interact()
            .map_err(|e| DBArenaError::Other(format!("Failed to read confirmation: {}", e)))?;

        if !confirmed {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    println!(
        "{} Deleting template {}...",
        style("→").cyan(),
        style(&name).bold()
    );

    template_manager.delete(&name)?;

    println!("  {} Template deleted successfully", style("✓").green());

    Ok(())
}

pub async fn handle_template_export(name: String, path: PathBuf) -> Result<()> {
    let template_manager = TemplateManager::new()?;

    println!(
        "{} Exporting template {} to {}...",
        style("→").cyan(),
        style(&name).bold(),
        style(path.display()).bold()
    );

    template_manager.export(&name, &path)?;

    println!("  {} Template exported successfully", style("✓").green());

    Ok(())
}

pub async fn handle_template_import(path: PathBuf) -> Result<()> {
    let template_manager = TemplateManager::new()?;

    println!(
        "{} Importing template from {}...",
        style("→").cyan(),
        style(path.display()).bold()
    );

    let template = template_manager.import(&path)?;

    println!("  {} Template imported successfully\n", style("✓").green());
    println!("  Name:     {}", template.name);
    println!("  Database: {}", template.database);

    Ok(())
}

pub async fn handle_template_inspect(name: String, json: bool) -> Result<()> {
    let template_manager = TemplateManager::new()?;
    let template = template_manager.load(&name)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&template)
                .map_err(|e| DBArenaError::Other(format!("JSON serialization failed: {}", e)))?
        );
        return Ok(());
    }

    println!("{} {}", style("Template:").cyan().bold(), style(&template.name).bold());
    println!();
    println!("  Database:    {}", template.database);
    if let Some(desc) = &template.description {
        println!("  Description: {}", desc);
    }

    if let Some(version) = &template.config.version {
        println!("\n  Version:     {}", version);
    }
    if let Some(port) = template.config.port {
        println!("  Port:        {}", port);
    }
    if let Some(persistent) = template.config.persistent {
        println!("  Persistent:  {}", persistent);
    }
    if let Some(memory) = template.config.memory_limit {
        println!("  Memory:      {} MB", memory / (1024 * 1024));
    }

    if !template.config.env_vars.is_empty() {
        println!("\n  Environment Variables:");
        for (key, value) in &template.config.env_vars {
            println!("    {} = {}", key, value);
        }
    }

    if !template.config.init_scripts.is_empty() {
        println!("\n  Init Scripts:");
        for script in &template.config.init_scripts {
            println!("    - {}", script);
        }
    }

    Ok(())
}
