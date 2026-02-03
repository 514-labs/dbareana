use serde::Deserialize;
use std::collections::HashMap;

/// Main seeding configuration
#[derive(Debug, Clone, Deserialize)]
pub struct SeedConfig {
    #[serde(default)]
    pub global_seed: Option<u64>,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default)]
    pub seed_rules: SeedRules,
}

fn default_batch_size() -> usize {
    1000
}

/// Wrapper for seed rules to support both array and nested structure
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum SeedRules {
    Flat(Vec<SeedRule>),
    Nested { tables: Vec<SeedRule> },
}

impl Default for SeedRules {
    fn default() -> Self {
        SeedRules::Flat(Vec::new())
    }
}

impl SeedRules {
    pub fn tables(&self) -> &[SeedRule] {
        match self {
            SeedRules::Flat(tables) => tables,
            SeedRules::Nested { tables } => tables,
        }
    }

    pub fn tables_mut(&mut self) -> &mut Vec<SeedRule> {
        match self {
            SeedRules::Flat(tables) => tables,
            SeedRules::Nested { tables } => tables,
        }
    }
}

/// Configuration for seeding a single table
#[derive(Debug, Clone, Deserialize)]
pub struct SeedRule {
    #[serde(alias = "table")]
    pub name: String,
    pub count: usize,
    pub columns: Vec<ColumnRule>,
}

/// Configuration for a single column's data generation
#[derive(Debug, Clone, Deserialize)]
pub struct ColumnRule {
    pub name: String,
    pub generator: String,
    #[serde(flatten)]
    pub options: HashMap<String, toml::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flat_config() {
        let toml = r#"
            global_seed = 42
            batch_size = 500

            [[seed_rules]]
            table = "users"
            count = 100

            [[seed_rules.columns]]
            name = "id"
            generator = "sequential"

            [[seed_rules.columns]]
            name = "email"
            generator = "email"
        "#;

        let config: SeedConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.global_seed, Some(42));
        assert_eq!(config.batch_size, 500);
        assert_eq!(config.seed_rules.tables().len(), 1);
        assert_eq!(config.seed_rules.tables()[0].name, "users");
        assert_eq!(config.seed_rules.tables()[0].count, 100);
        assert_eq!(config.seed_rules.tables()[0].columns.len(), 2);
    }

    #[test]
    fn test_parse_nested_config() {
        let toml = r#"
            global_seed = 42

            [seed_rules]
            [[seed_rules.tables]]
            name = "users"
            count = 100

            [[seed_rules.tables.columns]]
            name = "id"
            generator = "sequential"
        "#;

        let config: SeedConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.seed_rules.tables().len(), 1);
        assert_eq!(config.seed_rules.tables()[0].name, "users");
    }

    #[test]
    fn test_default_batch_size() {
        let toml = r#"
            [[seed_rules]]
            table = "users"
            count = 100

            [[seed_rules.columns]]
            name = "id"
            generator = "sequential"
        "#;

        let config: SeedConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.batch_size, 1000);
    }
}
