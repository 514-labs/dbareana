use crate::container::{ContainerManager, DockerClient};
use crate::{DBArenaError, Result};
use bollard::exec::{CreateExecOptions, StartExecResults};
use console::style;
use futures::StreamExt;
use std::io::Write;

pub async fn handle_exec(
    containers: Vec<String>,
    all: bool,
    filter: Option<String>,
    user: Option<String>,
    workdir: Option<String>,
    parallel: bool,
    command: Vec<String>,
) -> Result<()> {
    if command.is_empty() {
        return Err(DBArenaError::InvalidConfig(
            "Command is required. Use -- to separate container names from command.\nExample: dbarena exec postgres-1 -- echo hello".to_string(),
        ));
    }

    let docker_client = DockerClient::new()?;
    docker_client.verify_connection().await?;

    let manager = ContainerManager::new(docker_client.clone());

    // Resolve target containers
    let target_containers = get_target_containers(&manager, containers, all, filter).await?;

    if target_containers.is_empty() {
        return Err(DBArenaError::InvalidConfig(
            "No containers found matching the criteria".to_string(),
        ));
    }

    println!(
        "{} Executing command on {} container(s): {}",
        style("→").cyan(),
        style(target_containers.len()).bold(),
        style(command.join(" ")).yellow()
    );
    println!();

    if parallel && target_containers.len() > 1 {
        execute_parallel(&docker_client, &target_containers, &command, user.as_deref(), workdir.as_deref()).await
    } else {
        execute_sequential(&docker_client, &target_containers, &command, user.as_deref(), workdir.as_deref()).await
    }
}

async fn get_target_containers(
    manager: &ContainerManager,
    containers: Vec<String>,
    all: bool,
    filter: Option<String>,
) -> Result<Vec<(String, String)>> {
    let all_containers = manager.list_containers(false).await?;

    if all {
        // Get all running containers
        Ok(all_containers
            .into_iter()
            .map(|c| (c.id, c.name))
            .collect())
    } else if let Some(pattern) = filter {
        // Filter by glob pattern
        let pattern = glob::Pattern::new(&pattern)
            .map_err(|e| DBArenaError::InvalidConfig(format!("Invalid pattern: {}", e)))?;

        Ok(all_containers
            .into_iter()
            .filter(|c| pattern.matches(&c.name))
            .map(|c| (c.id, c.name))
            .collect())
    } else if !containers.is_empty() {
        // Resolve specific container names/IDs
        let mut resolved = Vec::new();
        for name_or_id in containers {
            let container = manager
                .find_container(&name_or_id)
                .await?
                .ok_or_else(|| DBArenaError::ContainerNotFound(name_or_id.clone()))?;
            resolved.push((container.id, container.name));
        }
        Ok(resolved)
    } else {
        Err(DBArenaError::InvalidConfig(
            "Must specify containers, --all, or --filter".to_string(),
        ))
    }
}

async fn execute_sequential(
    docker_client: &DockerClient,
    containers: &[(String, String)],
    command: &[String],
    user: Option<&str>,
    workdir: Option<&str>,
) -> Result<()> {
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for (id, name) in containers {
        println!("{} {}", style("Container:").cyan(), style(name).bold());
        match execute_single(docker_client, id, command, user, workdir).await {
            Ok(exit_code) => {
                if exit_code == 0 {
                    println!("  {} Exit code: {}\n", style("✓").green(), exit_code);
                    successes.push(name.clone());
                } else {
                    println!("  {} Exit code: {}\n", style("✗").red(), exit_code);
                    failures.push((name.clone(), format!("Exit code: {}", exit_code)));
                }
            }
            Err(e) => {
                eprintln!("  {} Error: {}\n", style("✗").red(), e);
                failures.push((name.clone(), e.to_string()));
            }
        }
    }

    print_summary(&successes, &failures)?;
    Ok(())
}

async fn execute_parallel(
    docker_client: &DockerClient,
    containers: &[(String, String)],
    command: &[String],
    user: Option<&str>,
    workdir: Option<&str>,
) -> Result<()> {
    use futures::future::join_all;

    let futures: Vec<_> = containers
        .iter()
        .map(|(id, name)| {
            let docker = docker_client.clone();
            let id = id.clone();
            let name = name.clone();
            let cmd = command.to_vec();
            let user = user.map(|s| s.to_string());
            let workdir = workdir.map(|s| s.to_string());

            async move {
                let result = execute_single(
                    &docker,
                    &id,
                    &cmd,
                    user.as_deref(),
                    workdir.as_deref(),
                )
                .await;
                (name, result)
            }
        })
        .collect();

    let results = join_all(futures).await;

    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for (name, result) in results {
        println!("{} {}", style("Container:").cyan(), style(&name).bold());
        match result {
            Ok(exit_code) => {
                if exit_code == 0 {
                    println!("  {} Exit code: {}\n", style("✓").green(), exit_code);
                    successes.push(name);
                } else {
                    println!("  {} Exit code: {}\n", style("✗").red(), exit_code);
                    failures.push((name, format!("Exit code: {}", exit_code)));
                }
            }
            Err(e) => {
                eprintln!("  {} Error: {}\n", style("✗").red(), e);
                failures.push((name, e.to_string()));
            }
        }
    }

    print_summary(&successes, &failures)?;
    Ok(())
}

fn print_summary(successes: &[String], failures: &[(String, String)]) -> Result<()> {
    let total = successes.len() + failures.len();

    // Only show summary if executing on multiple containers
    if total <= 1 {
        return Ok(());
    }

    println!("{}", "─".repeat(80));
    println!("{}", style("Execution Summary").bold());
    println!("{}", "─".repeat(80));

    if !successes.is_empty() {
        println!("{} {} container(s) succeeded:", style("✓").green(), successes.len());
        for name in successes {
            println!("  • {}", style(name).green());
        }
        println!();
    }

    if !failures.is_empty() {
        println!("{} {} container(s) failed:", style("✗").red(), failures.len());
        for (name, error) in failures {
            println!("  • {} - {}", style(name).red(), style(error).dim());
        }
        println!();
    }

    println!("{}/{} successful",
        style(successes.len()).bold(),
        style(total).bold()
    );

    // Return error if any failures occurred
    if !failures.is_empty() {
        return Err(DBArenaError::ContainerOperationFailed(
            format!("{} container(s) failed to execute command", failures.len())
        ));
    }

    Ok(())
}

async fn execute_single(
    docker_client: &DockerClient,
    container_id: &str,
    command: &[String],
    user: Option<&str>,
    workdir: Option<&str>,
) -> Result<i64> {
    let docker = docker_client.docker();

    // Create exec instance
    let exec_config = CreateExecOptions {
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        cmd: Some(command.to_vec()),
        user: user.map(|s| s.to_string()),
        working_dir: workdir.map(|s| s.to_string()),
        ..Default::default()
    };

    let exec = docker
        .create_exec(container_id, exec_config)
        .await
        .map_err(|e| DBArenaError::ContainerOperationFailed(format!("Failed to create exec: {}", e)))?;

    // Start exec and capture output
    let start_exec = docker
        .start_exec(&exec.id, None)
        .await
        .map_err(|e| DBArenaError::ContainerOperationFailed(format!("Failed to start exec: {}", e)))?;

    match start_exec {
        StartExecResults::Attached { mut output, .. } => {
            // Stream output
            while let Some(chunk) = output.next().await {
                match chunk {
                    Ok(bollard::container::LogOutput::StdOut { message }) => {
                        std::io::stdout().write_all(&message)?;
                        std::io::stdout().flush()?;
                    }
                    Ok(bollard::container::LogOutput::StdErr { message }) => {
                        std::io::stderr().write_all(&message)?;
                        std::io::stderr().flush()?;
                    }
                    Ok(bollard::container::LogOutput::Console { message }) => {
                        std::io::stdout().write_all(&message)?;
                        std::io::stdout().flush()?;
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

    Ok(inspect.exit_code.unwrap_or(0))
}
