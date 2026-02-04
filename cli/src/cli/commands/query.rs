use crate::cli::interactive;
use crate::container::{ContainerManager, DatabaseType, DockerClient};
use crate::{DBArenaError, Result};
use bollard::exec::{CreateExecOptions, StartExecResults};
use console::style;
use futures::StreamExt;
use std::path::PathBuf;

pub async fn handle_query(
    container: Option<String>,
    interactive_mode: bool,
    script: Option<String>,
    file: Option<PathBuf>,
) -> Result<()> {
    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client.clone());

    // Get container name
    let container_name = if interactive_mode {
        let all_containers = manager.list_containers(false).await?;
        interactive::select_container(all_containers, "query")?
    } else {
        container.ok_or_else(|| {
            DBArenaError::InvalidConfig(
                "Container name required. Use --container <name> or -i for interactive mode.".to_string(),
            )
        })?
    };

    // Find the container
    let found = manager
        .find_container(&container_name)
        .await?
        .ok_or_else(|| DBArenaError::ContainerNotFound(container_name.clone()))?;

    // Get database type
    let db_type = DatabaseType::from_string(&found.database_type)
        .ok_or_else(|| {
            DBArenaError::InvalidConfig(format!("Unknown database type: {}", found.database_type))
        })?;

    // Determine what SQL to execute
    let (sql_content, is_from_file) = if let Some(file_path) = file {
        // Read SQL from file
        let content = std::fs::read_to_string(&file_path).map_err(|e| {
            DBArenaError::InvalidConfig(format!("Failed to read file {:?}: {}", file_path, e))
        })?;
        (content, true)
    } else if let Some(sql) = script {
        (sql, false)
    } else {
        return Err(DBArenaError::InvalidConfig(
            "Either --script or --file must be provided".to_string(),
        ));
    };

    println!(
        "{} Executing {} on {}...",
        style("→").cyan(),
        if is_from_file { "SQL file" } else { "query" },
        style(&found.name).bold()
    );

    // Build command based on database type
    let cmd = build_query_command(db_type, &sql_content, is_from_file);

    // Execute the query
    let docker = docker_client.docker();
    let exec = docker
        .create_exec(
            &found.id,
            CreateExecOptions {
                cmd: Some(cmd),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await
        .map_err(|e| DBArenaError::ContainerOperationFailed(format!("Failed to create exec: {}", e)))?;

    // Start exec and stream output
    let start_exec = docker
        .start_exec(&exec.id, None)
        .await
        .map_err(|e| DBArenaError::ContainerOperationFailed(format!("Failed to start exec: {}", e)))?;

    println!();

    let mut output = String::new();
    match start_exec {
        StartExecResults::Attached { output: mut stream, .. } => {
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bollard::container::LogOutput::StdOut { message }) => {
                        let text = String::from_utf8_lossy(&message);
                        print!("{}", text);
                        output.push_str(&text);
                    }
                    Ok(bollard::container::LogOutput::StdErr { message }) => {
                        let text = String::from_utf8_lossy(&message);
                        eprint!("{}", text);
                        output.push_str(&text);
                    }
                    Ok(bollard::container::LogOutput::Console { message }) => {
                        let text = String::from_utf8_lossy(&message);
                        print!("{}", text);
                        output.push_str(&text);
                    }
                    Ok(bollard::container::LogOutput::StdIn { .. }) => {}
                    Err(e) => {
                        return Err(DBArenaError::ContainerOperationFailed(format!(
                            "Error reading output: {}",
                            e
                        )));
                    }
                }
            }
        }
        StartExecResults::Detached => {
            return Err(DBArenaError::ContainerOperationFailed(
                "Unexpected detached exec".to_string(),
            ));
        }
    }

    // Get exit code
    let inspect = docker
        .inspect_exec(&exec.id)
        .await
        .map_err(|e| DBArenaError::ContainerOperationFailed(format!("Failed to inspect exec: {}", e)))?;

    let exit_code = inspect.exit_code.unwrap_or(0);

    println!();
    if exit_code == 0 {
        println!("{} Query executed successfully", style("✓").green());
    } else {
        println!(
            "{} Query failed with exit code {}",
            style("✗").red(),
            exit_code
        );
        return Err(DBArenaError::ContainerOperationFailed(format!(
            "Query execution failed: {}",
            output
        )));
    }

    Ok(())
}

fn build_query_command(db_type: DatabaseType, sql: &str, is_file: bool) -> Vec<String> {
    match db_type {
        DatabaseType::Postgres => {
            if is_file {
                // For file content, write to temp file in container and execute
                vec![
                    "sh".to_string(),
                    "-c".to_string(),
                    format!("echo '{}' | psql -U postgres -d postgres", sql.replace('\'', "'\\'''")),
                ]
            } else {
                // For inline script, use -c flag
                vec![
                    "psql".to_string(),
                    "-U".to_string(),
                    "postgres".to_string(),
                    "-d".to_string(),
                    "postgres".to_string(),
                    "-c".to_string(),
                    sql.to_string(),
                ]
            }
        }
        DatabaseType::MySQL => {
            vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("echo '{}' | mysql -u root -pmysql", sql.replace('\'', "'\\'''")),
            ]
        }
        DatabaseType::SQLServer => {
            vec![
                "sh".to_string(),
                "-c".to_string(),
                format!(
                    "echo '{}' | /opt/mssql-tools/bin/sqlcmd -S localhost -U sa -P 'YourStrong@Passw0rd'",
                    sql.replace('\'', "'\\'''")
                ),
            ]
        }
    }
}
