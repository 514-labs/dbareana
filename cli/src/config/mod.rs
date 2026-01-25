//! Configuration management for dbarena
//!
//! This module provides configuration file loading, validation, and profile management.
//!
//! # Configuration Files
//!
//! dbarena supports TOML and YAML configuration files. Files are loaded in the following order:
//! 1. Explicit path via `--config` flag
//! 2. `./dbarena.toml` or `./dbarena.yaml` (project-local)
//! 3. `~/.config/dbarena/config.toml` or `config.yaml` (user config)
//!
//! # Example Configuration
//!
//! ```toml
//! [defaults]
//! persistent = false
//! memory_mb = 512
//!
//! [profiles.dev]
//! env = { LOG_LEVEL = "debug" }
//!
//! [databases.postgres.env]
//! POSTGRES_USER = "appuser"
//! POSTGRES_PASSWORD = "dev123"
//!
//! [databases.postgres.profiles.dev]
//! env = { POSTGRES_DB = "myapp_dev" }
//! ```

pub mod loader;
pub mod merger;
pub mod profile;
pub mod schema;
pub mod template;
pub mod validator;

pub use loader::{find_config_file, load_config, load_config_from_string, load_or_default, ConfigFormat};
pub use merger::{apply_cli_overrides, merge_configs, merge_env_vars};
pub use profile::{get_database_env, list_profiles, resolve_profile};
pub use schema::{DBArenaConfig, DatabaseConfig, DefaultsConfig, InitScript, ProfileConfig};
pub use template::{Template, TemplateConfig, TemplateManager};
pub use validator::{validate_config, validate_init_script_paths, ValidationResult};
