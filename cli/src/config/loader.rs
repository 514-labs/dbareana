use super::schema::DBArenaConfig;
use crate::error::{DBArenaError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Format of the configuration file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Toml,
    Yaml,
}

impl ConfigFormat {
    /// Detect format from file extension
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "toml" => Some(ConfigFormat::Toml),
                "yaml" | "yml" => Some(ConfigFormat::Yaml),
                _ => None,
            })
    }
}

/// Find configuration file using the precedence rules
///
/// Priority order:
/// 1. `./dbarena.toml` (project-local)
/// 2. `./dbarena.yaml` (project-local, alternate format)
/// 3. `$XDG_CONFIG_HOME/dbarena/config.toml` (user config, typically ~/.config/dbarena/)
/// 4. `$XDG_CONFIG_HOME/dbarena/config.yaml` (user config, alternate format)
///
/// Returns None if no config file is found (will use defaults)
pub fn find_config_file() -> Result<Option<PathBuf>> {
    // Check project-local configs first
    let local_toml = PathBuf::from("./dbarena.toml");
    if local_toml.exists() {
        return Ok(Some(local_toml));
    }

    let local_yaml = PathBuf::from("./dbarena.yaml");
    if local_yaml.exists() {
        return Ok(Some(local_yaml));
    }

    // Check user config directory (XDG_CONFIG_HOME or ~/.config)
    if let Some(config_dir) = dirs::config_dir() {
        let user_config_dir = config_dir.join("dbarena");

        let user_toml = user_config_dir.join("config.toml");
        if user_toml.exists() {
            return Ok(Some(user_toml));
        }

        let user_yaml = user_config_dir.join("config.yaml");
        if user_yaml.exists() {
            return Ok(Some(user_yaml));
        }
    }

    // No config file found - will use defaults
    Ok(None)
}

/// Load configuration from a file
pub fn load_config(path: impl AsRef<Path>) -> Result<DBArenaConfig> {
    let path = path.as_ref();

    // Read file contents
    let content = fs::read_to_string(path).map_err(|e| {
        DBArenaError::ConfigError(format!(
            "Failed to read config file '{}': {}",
            path.display(),
            e
        ))
    })?;

    // Detect format from extension
    let format = ConfigFormat::from_path(path).ok_or_else(|| {
        DBArenaError::ConfigError(format!(
            "Unknown config file format for '{}'. Use .toml, .yaml, or .yml extension",
            path.display()
        ))
    })?;

    // Parse based on format
    load_config_from_string(&content, format).map_err(|e| {
        DBArenaError::ConfigError(format!(
            "Failed to parse config file '{}': {}",
            path.display(),
            e
        ))
    })
}

/// Load configuration from a string with specified format
pub fn load_config_from_string(content: &str, format: ConfigFormat) -> Result<DBArenaConfig> {
    match format {
        ConfigFormat::Toml => {
            toml::from_str(content).map_err(|e| DBArenaError::ConfigError(format!("TOML parse error: {}", e)))
        }
        ConfigFormat::Yaml => {
            serde_yaml::from_str(content).map_err(|e| DBArenaError::ConfigError(format!("YAML parse error: {}", e)))
        }
    }
}

/// Load configuration with fallback to default
///
/// If explicit_path is provided, loads from that path (error if not found).
/// Otherwise, searches for config file using find_config_file().
/// Returns default config if no file is found.
pub fn load_or_default(explicit_path: Option<PathBuf>) -> Result<DBArenaConfig> {
    if let Some(path) = explicit_path {
        // Explicit path provided - must exist
        load_config(path)
    } else {
        // Search for config file
        if let Some(path) = find_config_file()? {
            load_config(path)
        } else {
            // No config file found - use defaults
            Ok(DBArenaConfig::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(
            ConfigFormat::from_path(Path::new("config.toml")),
            Some(ConfigFormat::Toml)
        );
        assert_eq!(
            ConfigFormat::from_path(Path::new("config.yaml")),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::from_path(Path::new("config.yml")),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(ConfigFormat::from_path(Path::new("config.txt")), None);
    }

    #[test]
    fn test_parse_toml() {
        let toml = r#"
            [defaults]
            persistent = false
            memory_mb = 512
        "#;

        let config = load_config_from_string(toml, ConfigFormat::Toml).unwrap();
        assert_eq!(config.defaults.memory_mb, Some(512));
    }

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
defaults:
  persistent: false
  memory_mb: 512
        "#;

        let config = load_config_from_string(yaml, ConfigFormat::Yaml).unwrap();
        assert_eq!(config.defaults.memory_mb, Some(512));
    }

    #[test]
    fn test_invalid_toml() {
        let toml = r#"
            [defaults
            persistent = false
        "#;

        let result = load_config_from_string(toml, ConfigFormat::Toml);
        assert!(result.is_err());
    }
}
