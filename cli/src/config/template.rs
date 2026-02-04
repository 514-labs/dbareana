use crate::container::{ContainerConfig, DatabaseType, VolumeMount};
use crate::{DBArenaError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Container template for reuse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: Option<String>,
    pub database: String,
    pub config: TemplateConfig,
}

/// Template configuration (serializable subset of ContainerConfig)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub database: String,
    pub version: Option<String>,
    pub port: Option<u16>,
    pub persistent: Option<bool>,
    pub memory_limit: Option<u64>,
    pub cpu_shares: Option<u64>,

    #[serde(default)]
    pub env_vars: HashMap<String, String>,

    #[serde(default)]
    pub init_scripts: Vec<String>,

    #[serde(default)]
    pub volumes: Vec<VolumeMount>,

    #[serde(default)]
    pub network: Option<String>,

    #[serde(default)]
    pub network_aliases: Vec<String>,
}

impl Template {
    pub fn from_container_config(
        name: String,
        description: Option<String>,
        config: &ContainerConfig,
    ) -> Self {
        Self {
            name: name.clone(),
            description,
            database: config.database.as_str().to_string(),
            config: TemplateConfig {
                database: config.database.as_str().to_string(),
                version: Some(config.version.clone()),
                port: config.port,
                persistent: Some(config.persistent),
                memory_limit: config.memory_limit,
                cpu_shares: config.cpu_shares,
                env_vars: config.env_vars.clone(),
                init_scripts: config
                    .init_scripts
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect(),
                volumes: config.volumes.clone(),
                network: None, // TODO: get from container
                network_aliases: Vec::new(), // TODO: get from container
            },
        }
    }

    pub fn to_container_config(&self, name: Option<String>) -> Result<ContainerConfig> {
        let database = DatabaseType::from_string(&self.config.database).ok_or_else(|| {
            DBArenaError::InvalidConfig(format!("Invalid database type: {}", self.config.database))
        })?;

        Ok(ContainerConfig {
            name,
            database,
            version: self
                .config
                .version
                .clone()
                .unwrap_or_else(|| "latest".to_string()),
            port: self.config.port,
            persistent: self.config.persistent.unwrap_or(false),
            memory_limit: self.config.memory_limit,
            cpu_shares: self.config.cpu_shares,
            env_vars: self.config.env_vars.clone(),
            init_scripts: self
                .config
                .init_scripts
                .iter()
                .map(PathBuf::from)
                .collect(),
            continue_on_error: false,
            volumes: self.config.volumes.clone(),
        })
    }
}

/// Template storage and management
pub struct TemplateManager {
    storage_path: PathBuf,
}

impl TemplateManager {
    pub fn new() -> Result<Self> {
        let storage_path = Self::default_storage_path()?;
        fs::create_dir_all(&storage_path)?;
        Ok(Self { storage_path })
    }

    pub fn with_path(storage_path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&storage_path)?;
        Ok(Self { storage_path })
    }

    fn default_storage_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            DBArenaError::InvalidConfig("Could not determine home directory".to_string())
        })?;

        Ok(home
            .join(".local")
            .join("share")
            .join("dbarena")
            .join("templates"))
    }

    fn template_path(&self, name: &str) -> PathBuf {
        self.storage_path.join(format!("{}.toml", name))
    }

    /// Save a template
    pub fn save(&self, template: &Template) -> Result<()> {
        let path = self.template_path(&template.name);
        let content = toml::to_string_pretty(template)
            .map_err(|e| DBArenaError::Other(format!("Failed to serialize template: {}", e)))?;

        fs::write(&path, content).map_err(|e| {
            DBArenaError::Other(format!(
                "Failed to write template to {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Load a template by name
    pub fn load(&self, name: &str) -> Result<Template> {
        let path = self.template_path(name);
        if !path.exists() {
            return Err(DBArenaError::InvalidConfig(format!(
                "Template '{}' not found",
                name
            )));
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            DBArenaError::Other(format!(
                "Failed to read template from {}: {}",
                path.display(),
                e
            ))
        })?;

        let template: Template = toml::from_str(&content).map_err(|e| {
            DBArenaError::Other(format!("Failed to parse template {}: {}", name, e))
        })?;

        Ok(template)
    }

    /// List all templates
    pub fn list(&self) -> Result<Vec<Template>> {
        if !self.storage_path.exists() {
            return Ok(Vec::new());
        }

        let mut templates = Vec::new();

        for entry in fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    match self.load(name) {
                        Ok(template) => templates.push(template),
                        Err(e) => {
                            eprintln!("Warning: Failed to load template {}: {}", name, e);
                        }
                    }
                }
            }
        }

        Ok(templates)
    }

    /// Delete a template
    pub fn delete(&self, name: &str) -> Result<()> {
        let path = self.template_path(name);
        if !path.exists() {
            return Err(DBArenaError::InvalidConfig(format!(
                "Template '{}' not found",
                name
            )));
        }

        fs::remove_file(&path).map_err(|e| {
            DBArenaError::Other(format!(
                "Failed to delete template {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Export a template to a file
    pub fn export(&self, name: &str, dest: &PathBuf) -> Result<()> {
        let template = self.load(name)?;
        let content = toml::to_string_pretty(&template)
            .map_err(|e| DBArenaError::Other(format!("Failed to serialize template: {}", e)))?;

        fs::write(dest, content).map_err(|e| {
            DBArenaError::Other(format!(
                "Failed to write template to {}: {}",
                dest.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Import a template from a file
    pub fn import(&self, path: &PathBuf) -> Result<Template> {
        let content = fs::read_to_string(path).map_err(|e| {
            DBArenaError::Other(format!(
                "Failed to read template from {}: {}",
                path.display(),
                e
            ))
        })?;

        let template: Template = toml::from_str(&content).map_err(|e| {
            DBArenaError::Other(format!("Failed to parse template: {}", e))
        })?;

        // Save to storage
        self.save(&template)?;

        Ok(template)
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create template manager")
    }
}
