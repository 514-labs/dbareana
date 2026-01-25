//! Initialization script execution module
//!
//! This module handles copying and executing SQL initialization scripts
//! in database containers with comprehensive error reporting and logging.

pub mod copier;
pub mod executor;
pub mod logs;

pub use copier::{copy_file_to_container, copy_files_to_container};
pub use executor::{execute_init_scripts, ScriptError, ScriptResult};
pub use logs::{ExecutionMetadata, LogEntry, LogManager, LogSession, ScriptMetadata};
