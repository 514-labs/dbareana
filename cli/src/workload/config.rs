use serde::Deserialize;
use std::collections::HashMap;

/// Main workload configuration
#[derive(Debug, Clone, Deserialize)]
pub struct WorkloadConfig {
    pub name: String,

    /// Built-in workload pattern
    #[serde(default)]
    pub pattern: Option<WorkloadPattern>,

    /// Custom operation mix
    #[serde(default)]
    pub custom_operations: Option<CustomOperations>,

    /// Custom SQL queries
    #[serde(default)]
    pub custom_queries: Option<Vec<CustomQuery>>,

    /// Tables to operate on
    pub tables: Vec<String>,

    /// Number of concurrent connections/workers
    #[serde(default = "default_connections")]
    pub connections: usize,

    /// Target transactions per second
    #[serde(default = "default_target_tps")]
    pub target_tps: usize,

    /// Duration in seconds (optional)
    #[serde(default)]
    pub duration_seconds: Option<u64>,

    /// Total transaction count (optional)
    #[serde(default)]
    pub transaction_count: Option<u64>,
}

fn default_connections() -> usize {
    10
}

fn default_target_tps() -> usize {
    100
}

/// Built-in workload patterns
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadPattern {
    // Transactional patterns
    Oltp,
    Ecommerce,

    // Analytical patterns
    Olap,
    Reporting,

    // Specialized patterns
    TimeSeries,
    SocialMedia,
    Iot,

    // Generic patterns
    ReadHeavy,
    WriteHeavy,
    Balanced,
}

impl WorkloadPattern {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "oltp" => Some(Self::Oltp),
            "ecommerce" | "e-commerce" => Some(Self::Ecommerce),
            "olap" => Some(Self::Olap),
            "reporting" => Some(Self::Reporting),
            "timeseries" | "time-series" | "time_series" => Some(Self::TimeSeries),
            "socialmedia" | "social-media" | "social_media" => Some(Self::SocialMedia),
            "iot" => Some(Self::Iot),
            "readheavy" | "read-heavy" | "read_heavy" => Some(Self::ReadHeavy),
            "writeheavy" | "write-heavy" | "write_heavy" => Some(Self::WriteHeavy),
            "balanced" => Some(Self::Balanced),
            _ => None,
        }
    }

    pub fn operation_weights(&self) -> OperationWeights {
        match self {
            // Transactional
            Self::Oltp => OperationWeights {
                select: 0.40,
                insert: 0.30,
                update: 0.25,
                delete: 0.05,
            },
            Self::Ecommerce => OperationWeights {
                select: 0.50,
                insert: 0.25,
                update: 0.20,
                delete: 0.05,
            },

            // Analytical
            Self::Olap => OperationWeights {
                select: 0.90,
                insert: 0.05,
                update: 0.04,
                delete: 0.01,
            },
            Self::Reporting => OperationWeights {
                select: 0.95,
                insert: 0.03,
                update: 0.02,
                delete: 0.0,
            },

            // Specialized
            Self::TimeSeries => OperationWeights {
                select: 0.30,
                insert: 0.65,
                update: 0.02,
                delete: 0.03,
            },
            Self::SocialMedia => OperationWeights {
                select: 0.70,
                insert: 0.20,
                update: 0.08,
                delete: 0.02,
            },
            Self::Iot => OperationWeights {
                select: 0.20,
                insert: 0.75,
                update: 0.03,
                delete: 0.02,
            },

            // Generic
            Self::ReadHeavy => OperationWeights {
                select: 0.80,
                insert: 0.10,
                update: 0.08,
                delete: 0.02,
            },
            Self::WriteHeavy => OperationWeights {
                select: 0.20,
                insert: 0.40,
                update: 0.30,
                delete: 0.10,
            },
            Self::Balanced => OperationWeights {
                select: 0.50,
                insert: 0.25,
                update: 0.20,
                delete: 0.05,
            },
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Oltp => "OLTP: Frequent small transactions (banking, booking)",
            Self::Ecommerce => "E-commerce: Shopping carts, orders, inventory",
            Self::Olap => "OLAP: Complex analytical queries with joins",
            Self::Reporting => "Reporting: Read-heavy with large result sets",
            Self::TimeSeries => "Time-series: High-volume inserts, range queries",
            Self::SocialMedia => "Social media: High reads, burst writes, feeds",
            Self::Iot => "IoT: Sensor data ingestion and aggregation",
            Self::ReadHeavy => "Generic: 80% reads, 20% writes",
            Self::WriteHeavy => "Generic: 80% writes, 20% reads",
            Self::Balanced => "Generic: 50% reads, 50% writes",
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Oltp => "oltp",
            Self::Ecommerce => "ecommerce",
            Self::Olap => "olap",
            Self::Reporting => "reporting",
            Self::TimeSeries => "time_series",
            Self::SocialMedia => "social_media",
            Self::Iot => "iot",
            Self::ReadHeavy => "read_heavy",
            Self::WriteHeavy => "write_heavy",
            Self::Balanced => "balanced",
        }
    }
}

/// Operation weights for custom workloads
#[derive(Debug, Clone, Deserialize)]
pub struct OperationWeights {
    pub select: f64,
    pub insert: f64,
    pub update: f64,
    pub delete: f64,
}

impl OperationWeights {
    pub fn normalize(&mut self) {
        let total = self.select + self.insert + self.update + self.delete;
        if total > 0.0 {
            self.select /= total;
            self.insert /= total;
            self.update /= total;
            self.delete /= total;
        }
    }

    pub fn is_valid(&self) -> bool {
        let total = self.select + self.insert + self.update + self.delete;
        (total - 1.0).abs() < 0.001
    }
}

/// Custom operation configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CustomOperations {
    #[serde(default)]
    pub select_weight: f64,
    #[serde(default)]
    pub insert_weight: f64,
    #[serde(default)]
    pub update_weight: f64,
    #[serde(default)]
    pub delete_weight: f64,

    #[serde(default)]
    pub use_joins: bool,
    #[serde(default)]
    pub use_aggregations: bool,
    #[serde(default)]
    pub avg_result_set_size: Option<usize>,
}

impl CustomOperations {
    pub fn to_weights(&self) -> OperationWeights {
        let mut weights = OperationWeights {
            select: self.select_weight,
            insert: self.insert_weight,
            update: self.update_weight,
            delete: self.delete_weight,
        };
        weights.normalize();
        weights
    }
}

/// Custom SQL query configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CustomQuery {
    pub name: String,
    pub sql: String,
    pub weight: f64,

    #[serde(default)]
    pub parameters: Vec<QueryParameter>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryParameter {
    pub name: String,
    pub generator: String,
    #[serde(flatten)]
    pub options: HashMap<String, toml::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let toml = r#"
            name = "Test Workload"
            pattern = "oltp"
            tables = ["users", "orders"]
            connections = 20
            target_tps = 200
            duration_seconds = 60
        "#;

        let config: WorkloadConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.name, "Test Workload");
        assert_eq!(config.pattern, Some(WorkloadPattern::Oltp));
        assert_eq!(config.tables.len(), 2);
        assert_eq!(config.connections, 20);
        assert_eq!(config.target_tps, 200);
        assert_eq!(config.duration_seconds, Some(60));
    }

    #[test]
    fn test_pattern_from_str() {
        assert_eq!(
            WorkloadPattern::from_str("oltp"),
            Some(WorkloadPattern::Oltp)
        );
        assert_eq!(
            WorkloadPattern::from_str("ecommerce"),
            Some(WorkloadPattern::Ecommerce)
        );
        assert_eq!(
            WorkloadPattern::from_str("read-heavy"),
            Some(WorkloadPattern::ReadHeavy)
        );
        assert!(WorkloadPattern::from_str("invalid").is_none());
    }

    #[test]
    fn test_operation_weights() {
        let oltp = WorkloadPattern::Oltp;
        let weights = oltp.operation_weights();

        assert_eq!(weights.select, 0.40);
        assert_eq!(weights.insert, 0.30);
        assert_eq!(weights.update, 0.25);
        assert_eq!(weights.delete, 0.05);

        let total = weights.select + weights.insert + weights.update + weights.delete;
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_weights_normalize() {
        let mut weights = OperationWeights {
            select: 10.0,
            insert: 5.0,
            update: 3.0,
            delete: 2.0,
        };

        weights.normalize();

        assert!((weights.select - 0.5).abs() < 0.001);
        assert!((weights.insert - 0.25).abs() < 0.001);
        assert!((weights.update - 0.15).abs() < 0.001);
        assert!((weights.delete - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_custom_operations_config() {
        let toml = r#"
            name = "Custom Workload"
            tables = ["users"]

            [custom_operations]
            select_weight = 0.7
            insert_weight = 0.2
            update_weight = 0.08
            delete_weight = 0.02
            use_joins = true
        "#;

        let config: WorkloadConfig = toml::from_str(toml).unwrap();
        assert!(config.custom_operations.is_some());

        let custom = config.custom_operations.unwrap();
        assert_eq!(custom.select_weight, 0.7);
        assert!(custom.use_joins);
    }

    #[test]
    fn test_custom_queries_config() {
        let toml = r#"
            name = "Custom SQL Workload"
            tables = ["orders"]

            [[custom_queries]]
            name = "recent_orders"
            sql = "SELECT * FROM orders WHERE created_at > NOW() - INTERVAL '1 hour'"
            weight = 0.5
        "#;

        let config: WorkloadConfig = toml::from_str(toml).unwrap();
        assert!(config.custom_queries.is_some());

        let queries = config.custom_queries.unwrap();
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].name, "recent_orders");
        assert_eq!(queries[0].weight, 0.5);
    }

    #[test]
    fn test_all_patterns_have_valid_weights() {
        let patterns = vec![
            WorkloadPattern::Oltp,
            WorkloadPattern::Ecommerce,
            WorkloadPattern::Olap,
            WorkloadPattern::Reporting,
            WorkloadPattern::TimeSeries,
            WorkloadPattern::SocialMedia,
            WorkloadPattern::Iot,
            WorkloadPattern::ReadHeavy,
            WorkloadPattern::WriteHeavy,
            WorkloadPattern::Balanced,
        ];

        for pattern in patterns {
            let weights = pattern.operation_weights();
            let total = weights.select + weights.insert + weights.update + weights.delete;
            assert!(
                (total - 1.0).abs() < 0.001,
                "Pattern {:?} has invalid weights (total: {})",
                pattern,
                total
            );
        }
    }
}
