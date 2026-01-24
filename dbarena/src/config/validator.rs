use super::schema::{DBArenaConfig, InitScript};
use crate::error::{DBArenaError, Result};
use std::collections::HashMap;
use std::path::Path;

/// Validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
    }

    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
}

/// Validate a configuration
pub fn validate_config(config: &DBArenaConfig) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();

    // Validate defaults
    validate_defaults(config, &mut result);

    // Validate environment variables in profiles
    validate_profiles(&config.profiles, &mut result);

    // Validate databases
    validate_databases(config, &mut result);

    // Return error if any validation errors were found
    if !result.is_valid() {
        return Err(DBArenaError::ConfigError(format!(
            "Configuration validation failed:\n{}",
            result.errors.join("\n")
        )));
    }

    Ok(result)
}

/// Validate default settings
fn validate_defaults(config: &DBArenaConfig, result: &mut ValidationResult) {
    if let Some(memory_mb) = config.defaults.memory_mb {
        if memory_mb == 0 {
            result.add_error("defaults.memory_mb must be greater than 0");
        }
        if memory_mb > 1024 * 1024 {
            result.add_warning(format!(
                "defaults.memory_mb is very large: {}MB ({}GB)",
                memory_mb,
                memory_mb / 1024
            ));
        }
    }

    if let Some(cpu_shares) = config.defaults.cpu_shares {
        if cpu_shares == 0 {
            result.add_error("defaults.cpu_shares must be greater than 0");
        }
    }
}

/// Validate environment variable profiles
fn validate_profiles(profiles: &HashMap<String, super::schema::ProfileConfig>, result: &mut ValidationResult) {
    for (name, profile) in profiles {
        validate_env_vars(&profile.env, &format!("profiles.{}", name), result);
    }
}

/// Validate database configurations
fn validate_databases(config: &DBArenaConfig, result: &mut ValidationResult) {
    for (db_name, db_config) in &config.databases {
        let prefix = format!("databases.{}", db_name);

        // Validate environment variables
        validate_env_vars(&db_config.env, &prefix, result);

        // Validate database-specific profiles
        for (profile_name, profile) in &db_config.profiles {
            validate_env_vars(
                &profile.env,
                &format!("{}.profiles.{}", prefix, profile_name),
                result,
            );

            // Check if global profile exists (warning only)
            if !config.profiles.contains_key(profile_name) {
                result.add_warning(format!(
                    "Database '{}' defines profile '{}' but no global profile with this name exists",
                    db_name, profile_name
                ));
            }
        }

        // Validate init scripts (paths will be validated later when actually used)
        for (idx, script) in db_config.init_scripts.iter().enumerate() {
            validate_init_script(script, &format!("{}.init_scripts[{}]", prefix, idx), result);
        }
    }
}

/// Validate environment variables
fn validate_env_vars(env: &HashMap<String, String>, context: &str, result: &mut ValidationResult) {
    for (key, value) in env {
        // Key validation
        if key.is_empty() {
            result.add_error(format!("{}: Environment variable key cannot be empty", context));
            continue;
        }

        if key.contains('=') {
            result.add_error(format!(
                "{}: Environment variable key '{}' cannot contain '='",
                context, key
            ));
        }

        if key.contains(' ') {
            result.add_error(format!(
                "{}: Environment variable key '{}' cannot contain spaces",
                context, key
            ));
        }

        if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            result.add_warning(format!(
                "{}: Environment variable key '{}' contains unusual characters. Typically keys use only A-Z, 0-9, and underscore.",
                context, key
            ));
        }

        // Value validation
        if value.is_empty() {
            result.add_warning(format!(
                "{}: Environment variable '{}' has an empty value",
                context, key
            ));
        }
    }
}

/// Validate init script configuration
fn validate_init_script(script: &InitScript, context: &str, result: &mut ValidationResult) {
    let path = script.path();

    if path.is_empty() {
        result.add_error(format!("{}: Init script path cannot be empty", context));
        return;
    }

    // Check for glob patterns - these are valid but will be expanded later
    if path.contains('*') || path.contains('?') {
        // This is a glob pattern - valid but can't check existence yet
        return;
    }

    // For non-glob paths, we could check existence but paths might be relative
    // to config file location, so we'll skip this check here and do it when
    // the config is actually used
}

/// Validate that init script files exist
///
/// This is a separate validation step because it requires knowing the base path
/// for resolving relative paths (typically the directory containing the config file).
pub fn validate_init_script_paths(
    config: &DBArenaConfig,
    base_path: &Path,
) -> Result<ValidationResult> {
    let mut result = ValidationResult::new();

    for (db_name, db_config) in &config.databases {
        for (idx, script) in db_config.init_scripts.iter().enumerate() {
            let path = script.path();
            let context = format!("databases.{}.init_scripts[{}]", db_name, idx);

            // Skip glob patterns - they'll be expanded later
            if path.contains('*') || path.contains('?') {
                continue;
            }

            // Resolve relative paths
            let script_path = if Path::new(path).is_absolute() {
                Path::new(path).to_path_buf()
            } else {
                base_path.join(path)
            };

            // Check if file exists
            if !script_path.exists() {
                result.add_error(format!(
                    "{}: Init script file not found: {}",
                    context,
                    script_path.display()
                ));
            } else if !script_path.is_file() {
                result.add_error(format!(
                    "{}: Init script path is not a file: {}",
                    context,
                    script_path.display()
                ));
            }
        }
    }

    if !result.is_valid() {
        return Err(DBArenaError::ConfigError(format!(
            "Init script validation failed:\n{}",
            result.errors.join("\n")
        )));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_config() {
        let config = DBArenaConfig::default();
        let result = validate_config(&config).unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_invalid_memory() {
        let toml = r#"
            [defaults]
            memory_mb = 0
        "#;
        let config: DBArenaConfig = toml::from_str(toml).unwrap();
        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_env_key() {
        let toml = r#"
            [profiles.dev]
            env = { "INVALID KEY" = "value" }
        "#;
        let config: DBArenaConfig = toml::from_str(toml).unwrap();
        let result = validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_config() {
        let toml = r#"
            [defaults]
            memory_mb = 512

            [profiles.dev]
            env = { LOG_LEVEL = "debug" }

            [databases.postgres.env]
            POSTGRES_DB = "myapp"
        "#;
        let config: DBArenaConfig = toml::from_str(toml).unwrap();
        let result = validate_config(&config).unwrap();
        assert!(result.is_valid());
    }
}
