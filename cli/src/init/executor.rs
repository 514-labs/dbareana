use super::copier::copy_files_to_container;
use super::logs::{ExecutionMetadata, LogManager, ScriptMetadata};
use crate::container::{ContainerConfig, DatabaseType};
use crate::Result;
use bollard::Docker;
use bollard::exec::{CreateExecOptions, StartExecResults};
use futures::StreamExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Result of executing a single script
#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub script_path: PathBuf,
    pub success: bool,
    pub output: String,
    pub error: Option<ScriptError>,
    pub duration: Duration,
    pub statements_executed: usize,
}

/// Detailed script error information
#[derive(Debug, Clone)]
pub struct ScriptError {
    pub script_path: PathBuf,
    pub line_number: Option<usize>,
    pub column_number: Option<usize>,
    pub error_message: String,
    pub database_error_code: Option<String>,
    pub context_lines: Option<Vec<String>>,
    pub suggestion: Option<String>,
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error_message)?;
        if let Some(line) = self.line_number {
            write!(f, " at line {}", line)?;
        }
        if let Some(suggestion) = &self.suggestion {
            write!(f, "\nSuggestion: {}", suggestion)?;
        }
        Ok(())
    }
}

/// Execute initialization scripts in a container
pub async fn execute_init_scripts(
    docker: &Docker,
    container_id: &str,
    scripts: Vec<PathBuf>,
    db_type: DatabaseType,
    db_config: &ContainerConfig,
    continue_on_error: bool,
    log_manager: &LogManager,
) -> Result<Vec<ScriptResult>> {
    if scripts.is_empty() {
        return Ok(Vec::new());
    }

    // Create log session
    let session = log_manager.create_session(container_id)?;
    let start_time = Instant::now();
    let mut results = Vec::new();
    let mut success_count = 0;
    let mut failure_count = 0;

    // Copy all scripts to container
    // Note: Use /var instead of /tmp because tmpfs mounts prevent docker cp/upload
    let container_script_dir = "/var/dbarena_init";
    let script_refs: Vec<&Path> = scripts.iter().map(|p| p.as_path()).collect();
    copy_files_to_container(docker, container_id, &script_refs, container_script_dir).await?;

    // Execute each script
    for script_path in scripts {
        let script_name = script_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let container_script_path = format!("{}/{}", container_script_dir, script_name);

        let exec_start = Instant::now();
        let result = execute_single_script(
            docker,
            container_id,
            &script_path,
            &container_script_path,
            db_type,
            db_config,
            continue_on_error,
        )
        .await;

        let duration = exec_start.elapsed();

        match result {
            Ok((output, statements)) => {
                // Write log
                let _log_file = log_manager.write_script_log(&session, &script_path, &output)?;

                results.push(ScriptResult {
                    script_path: script_path.clone(),
                    success: true,
                    output,
                    error: None,
                    duration,
                    statements_executed: statements,
                });
                success_count += 1;
            }
            Err(e) => {
                let error_message = e.to_string();
                let output = error_message.clone();

                // Write error log
                let _log_file = log_manager.write_script_log(&session, &script_path, &output)?;

                // Parse error
                let script_error = parse_error(&error_message, &script_path, db_type);

                results.push(ScriptResult {
                    script_path: script_path.clone(),
                    success: false,
                    output,
                    error: Some(script_error),
                    duration,
                    statements_executed: 0,
                });
                failure_count += 1;

                // Stop if not continuing on error
                if !continue_on_error {
                    break;
                }
            }
        }
    }

    // Write metadata
    let total_duration = start_time.elapsed();
    let metadata = ExecutionMetadata {
        scripts: results
            .iter()
            .map(|r| {
                let log_file = session
                    .session_dir
                    .join(format!("{}.log", r.script_path.file_name().unwrap().to_str().unwrap()));
                ScriptMetadata {
                    path: r.script_path.clone(),
                    success: r.success,
                    duration: r.duration,
                    log_file,
                    error_summary: r.error.as_ref().map(|e| e.error_message.clone()),
                }
            })
            .collect(),
        total_duration,
        success_count,
        failure_count,
    };
    log_manager.write_metadata(&session, &metadata)?;

    Ok(results)
}

/// Execute a single SQL script
async fn execute_single_script(
    docker: &Docker,
    container_id: &str,
    _local_path: &Path,
    container_path: &str,
    db_type: DatabaseType,
    db_config: &ContainerConfig,
    continue_on_error: bool,
) -> Result<(String, usize)> {
    // Build command based on database type
    let cmd = build_exec_command(db_type, container_path, db_config, continue_on_error);

    // Create exec instance
    let exec = docker
        .create_exec(
            container_id,
            CreateExecOptions {
                cmd: Some(cmd),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await?;

    // Start exec and capture output
    let mut output = String::new();
    if let StartExecResults::Attached { output: mut stream, .. } =
        docker.start_exec(&exec.id, None).await?
    {
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    output.push_str(&chunk.to_string());
                }
                Err(e) => {
                    return Err(crate::DBArenaError::InitScriptFailed(format!(
                        "Failed to read exec output: {}",
                        e
                    )));
                }
            }
        }
    }

    // Check exit code
    let inspect = docker.inspect_exec(&exec.id).await?;
    let exit_code = inspect.exit_code.unwrap_or(1);

    // When continue_on_error is true, psql doesn't use ON_ERROR_STOP=1,
    // so it returns exit code 0 even with errors. Check output for errors.
    let has_errors = if continue_on_error {
        match db_type {
            DatabaseType::Postgres => output.contains("ERROR:"),
            DatabaseType::MySQL => output.contains("ERROR "),
            DatabaseType::SQLServer => output.contains("Msg "),
        }
    } else {
        false
    };

    if exit_code != 0 || has_errors {
        return Err(crate::DBArenaError::InitScriptFailed(format!(
            "Script failed with exit code {}: {}",
            exit_code, output
        )));
    }

    // Count statements (rough estimate)
    let statements = count_statements(&output);

    Ok((output, statements))
}

/// Build the exec command for running a script
fn build_exec_command(db_type: DatabaseType, script_path: &str, config: &ContainerConfig, continue_on_error: bool) -> Vec<String> {
    match db_type {
        DatabaseType::Postgres => {
            let user = config
                .env_vars
                .get("POSTGRES_USER")
                .map(|s| s.as_str())
                .unwrap_or("postgres");
            // Always use the postgres database for init scripts
            // The POSTGRES_DB env var is for the docker-entrypoint to create a database,
            // but that database might not be fully initialized when we run init scripts
            let db = "postgres";

            let mut cmd = vec![
                "psql".to_string(),
                "-U".to_string(),
                user.to_string(),
                "-d".to_string(),
                db.to_string(),
            ];

            // Only stop on error if continue_on_error is false
            if !continue_on_error {
                cmd.push("-v".to_string());
                cmd.push("ON_ERROR_STOP=1".to_string());
            }

            cmd.push("-f".to_string());
            cmd.push(script_path.to_string());

            cmd
        }
        DatabaseType::MySQL => {
            let password = config
                .env_vars
                .get("MYSQL_ROOT_PASSWORD")
                .map(|s| s.as_str())
                .unwrap_or("mysql");
            let db = config
                .env_vars
                .get("MYSQL_DATABASE")
                .map(|s| s.as_str())
                .unwrap_or("testdb");

            vec![
                "sh".to_string(),
                "-c".to_string(),
                format!(
                    "mysql -u root -p{} {} < {}",
                    password, db, script_path
                ),
            ]
        }
        DatabaseType::SQLServer => {
            let password = config
                .env_vars
                .get("SA_PASSWORD")
                .map(|s| s.as_str())
                .unwrap_or("YourStrong@Passw0rd");

            vec![
                "/opt/mssql-tools/bin/sqlcmd".to_string(),
                "-S".to_string(),
                "localhost".to_string(),
                "-U".to_string(),
                "sa".to_string(),
                "-P".to_string(),
                password.to_string(),
                "-i".to_string(),
                script_path.to_string(),
            ]
        }
    }
}

/// Parse error message and extract useful information
fn parse_error(error_msg: &str, script_path: &Path, db_type: DatabaseType) -> ScriptError {
    let mut script_error = ScriptError {
        script_path: script_path.to_path_buf(),
        line_number: None,
        column_number: None,
        error_message: error_msg.to_string(),
        database_error_code: None,
        context_lines: None,
        suggestion: None,
    };

    match db_type {
        DatabaseType::Postgres => {
            // Parse PostgreSQL errors
            // Format: ERROR:  syntax error at or near "INSRT" at character 1
            // LINE 1: INSRT INTO users ...
            if let Some(line_pos) = error_msg.find("LINE ") {
                if let Some(line_str) = error_msg[line_pos..].split(':').next() {
                    if let Some(num_str) = line_str.split_whitespace().nth(1) {
                        if let Ok(line_num) = num_str.parse::<usize>() {
                            script_error.line_number = Some(line_num);
                        }
                    }
                }
            }

            // Suggest common typos
            if error_msg.contains("INSRT") {
                script_error.suggestion = Some("Did you mean 'INSERT'?".to_string());
            } else if error_msg.contains("SLECT") {
                script_error.suggestion = Some("Did you mean 'SELECT'?".to_string());
            }
        }
        DatabaseType::MySQL => {
            // Parse MySQL errors
            // Format: ERROR 1064 (42000) at line 1: You have an error in your SQL syntax
            if let Some(line_pos) = error_msg.find("at line ") {
                if let Some(num_str) = error_msg[line_pos + 8..].split(':').next() {
                    if let Ok(line_num) = num_str.trim().parse::<usize>() {
                        script_error.line_number = Some(line_num);
                    }
                }
            }

            // Extract error code
            if let Some(code_start) = error_msg.find("ERROR ") {
                if let Some(code_str) = error_msg[code_start + 6..].split_whitespace().next() {
                    script_error.database_error_code = Some(code_str.to_string());
                }
            }
        }
        DatabaseType::SQLServer => {
            // Parse SQL Server errors
            // Format: Msg 102, Level 15, State 1, Server localhost, Line 1
            if let Some(line_pos) = error_msg.find("Line ") {
                if let Some(num_str) = error_msg[line_pos + 5..].split_whitespace().next() {
                    if let Ok(line_num) = num_str.parse::<usize>() {
                        script_error.line_number = Some(line_num);
                    }
                }
            }

            // Extract message number
            if let Some(msg_start) = error_msg.find("Msg ") {
                if let Some(msg_num) = error_msg[msg_start + 4..].split(',').next() {
                    script_error.database_error_code = Some(msg_num.trim().to_string());
                }
            }
        }
    }

    script_error
}

/// Count SQL statements in output (rough estimate)
fn count_statements(output: &str) -> usize {
    // Simple heuristic: count common success indicators
    let create_count = output.matches("CREATE").count();
    let insert_count = output.matches("INSERT").count();
    let update_count = output.matches("UPDATE").count();
    let delete_count = output.matches("DELETE").count();
    let alter_count = output.matches("ALTER").count();

    create_count + insert_count + update_count + delete_count + alter_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_postgres_error() {
        let error_msg = r#"ERROR:  syntax error at or near "INSRT"
LINE 1: INSRT INTO users (name) VALUES ('test');
        ^"#;
        let error = parse_error(error_msg, Path::new("/tmp/test.sql"), DatabaseType::Postgres);
        assert_eq!(error.line_number, Some(1));
        assert_eq!(error.suggestion, Some("Did you mean 'INSERT'?".to_string()));
    }

    #[test]
    fn test_parse_mysql_error() {
        let error_msg = "ERROR 1064 (42000) at line 5: You have an error in your SQL syntax";
        let error = parse_error(error_msg, Path::new("/tmp/test.sql"), DatabaseType::MySQL);
        assert_eq!(error.line_number, Some(5));
        assert_eq!(error.database_error_code, Some("1064".to_string()));
    }

    #[test]
    fn test_count_statements() {
        let output = "CREATE TABLE users;\nINSERT INTO users VALUES (1);\nINSERT INTO users VALUES (2);";
        assert_eq!(count_statements(output), 3);
    }

    #[test]
    fn test_build_postgres_command() {
        let config = ContainerConfig::new(DatabaseType::Postgres);
        let cmd = build_exec_command(DatabaseType::Postgres, "/tmp/test.sql", &config, false);
        assert_eq!(cmd[0], "psql");
        assert!(cmd.contains(&"-U".to_string()));
        assert!(cmd.contains(&"postgres".to_string()));
        assert!(cmd.contains(&"ON_ERROR_STOP=1".to_string()));

        // Test with continue_on_error = true
        let cmd_continue = build_exec_command(DatabaseType::Postgres, "/tmp/test.sql", &config, true);
        assert!(!cmd_continue.contains(&"ON_ERROR_STOP=1".to_string()));
    }
}
