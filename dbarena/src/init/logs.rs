use crate::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Manages logs for initialization scripts
pub struct LogManager {
    log_dir: PathBuf,
}

/// A log session for a specific container
#[derive(Debug, Clone)]
pub struct LogSession {
    pub container_id: String,
    pub session_dir: PathBuf,
    pub created_at: SystemTime,
}

/// Metadata for a single script execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMetadata {
    pub path: PathBuf,
    pub success: bool,
    pub duration: Duration,
    pub log_file: PathBuf,
    pub error_summary: Option<String>,
}

/// Metadata for the entire initialization session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub scripts: Vec<ScriptMetadata>,
    pub total_duration: Duration,
    pub success_count: usize,
    pub failure_count: usize,
}

/// A single log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub script_name: String,
    pub success: bool,
    pub log_path: PathBuf,
}

impl LogManager {
    /// Create a new LogManager
    ///
    /// Defaults to ~/.local/share/dbarena/logs/ or uses the specified directory
    pub fn new(custom_dir: Option<PathBuf>) -> Result<Self> {
        let log_dir = if let Some(dir) = custom_dir {
            dir
        } else {
            // Use XDG data directory
            let data_dir = dirs::data_local_dir().ok_or_else(|| {
                crate::DBArenaError::Other("Could not determine local data directory".to_string())
            })?;
            data_dir.join("dbarena").join("logs")
        };

        // Create log directory if it doesn't exist
        fs::create_dir_all(&log_dir).map_err(|e| {
            crate::DBArenaError::IoError(std::io::Error::new(
                e.kind(),
                format!("Failed to create log directory '{}': {}", log_dir.display(), e),
            ))
        })?;

        Ok(Self { log_dir })
    }

    /// Create a new log session for a container
    pub fn create_session(&self, container_id: &str) -> Result<LogSession> {
        let session_dir = self.log_dir.join(container_id);
        fs::create_dir_all(&session_dir)?;

        Ok(LogSession {
            container_id: container_id.to_string(),
            session_dir,
            created_at: SystemTime::now(),
        })
    }

    /// Write script output to a log file
    pub fn write_script_log(
        &self,
        session: &LogSession,
        script: &Path,
        output: &str,
    ) -> Result<PathBuf> {
        let script_name = script
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let log_file = session.session_dir.join(format!("{}.log", script_name));

        fs::write(&log_file, output)?;

        Ok(log_file)
    }

    /// Write execution metadata to a JSON file
    pub fn write_metadata(&self, session: &LogSession, metadata: &ExecutionMetadata) -> Result<()> {
        let metadata_file = session.session_dir.join("metadata.json");
        let json = serde_json::to_string_pretty(metadata)?;
        fs::write(metadata_file, json)?;
        Ok(())
    }

    /// Get all log entries for a container
    pub fn get_session_logs(&self, container_id: &str) -> Result<Vec<LogEntry>> {
        let session_dir = self.log_dir.join(container_id);
        if !session_dir.exists() {
            return Ok(Vec::new());
        }

        // Read metadata if it exists
        let metadata_file = session_dir.join("metadata.json");
        if !metadata_file.exists() {
            return Ok(Vec::new());
        }

        let metadata_content = fs::read_to_string(&metadata_file)?;
        let metadata: ExecutionMetadata = serde_json::from_str(&metadata_content)?;

        Ok(metadata
            .scripts
            .into_iter()
            .map(|script| LogEntry {
                script_name: script
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                success: script.success,
                log_path: script.log_file,
            })
            .collect())
    }

    /// Get the log directory path
    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_log_manager() {
        let temp_dir = TempDir::new().unwrap();
        let log_manager = LogManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
        assert!(log_manager.log_dir.exists());
    }

    #[test]
    fn test_create_session() {
        let temp_dir = TempDir::new().unwrap();
        let log_manager = LogManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
        let session = log_manager.create_session("test-container-123").unwrap();
        assert!(session.session_dir.exists());
        assert_eq!(session.container_id, "test-container-123");
    }

    #[test]
    fn test_write_script_log() {
        let temp_dir = TempDir::new().unwrap();
        let log_manager = LogManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
        let session = log_manager.create_session("test-container").unwrap();

        let script_path = PathBuf::from("/tmp/test.sql");
        let output = "CREATE TABLE test;\nINSERT INTO test VALUES (1);";

        let log_file = log_manager
            .write_script_log(&session, &script_path, output)
            .unwrap();

        assert!(log_file.exists());
        let content = fs::read_to_string(&log_file).unwrap();
        assert_eq!(content, output);
    }

    #[test]
    fn test_write_and_read_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let log_manager = LogManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
        let session = log_manager.create_session("test-container").unwrap();

        let metadata = ExecutionMetadata {
            scripts: vec![ScriptMetadata {
                path: PathBuf::from("/tmp/test.sql"),
                success: true,
                duration: Duration::from_secs(1),
                log_file: PathBuf::from("/tmp/test.sql.log"),
                error_summary: None,
            }],
            total_duration: Duration::from_secs(1),
            success_count: 1,
            failure_count: 0,
        };

        log_manager.write_metadata(&session, &metadata).unwrap();

        let logs = log_manager.get_session_logs("test-container").unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].script_name, "test.sql");
        assert!(logs[0].success);
    }
}
