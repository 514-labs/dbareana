use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::container::DatabaseType;

/// Database-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub container_id: String,
    pub database_type: DatabaseType,
    pub timestamp: i64,

    // Connection metrics
    pub active_connections: u64,
    pub max_connections: Option<u64>,
    pub connections_by_state: HashMap<String, u64>, // active, idle, idle_in_transaction

    // Query metrics
    pub queries_per_second: f64,
    pub query_breakdown: QueryBreakdown, // SELECT, INSERT, UPDATE, DELETE counts

    // Transaction metrics
    pub transactions_per_second: f64,
    pub commits_per_second: f64,
    pub rollbacks_per_second: f64,

    // Cache/Buffer metrics
    pub cache_hit_ratio: Option<f64>, // Percentage (0.0-100.0)

    // Replication metrics (optional)
    pub replication_lag_bytes: Option<u64>,
    pub replication_status: Option<String>,

    // Database-specific extras
    pub extras: HashMap<String, serde_json::Value>,
}

impl DatabaseMetrics {
    pub fn new(container_id: String, database_type: DatabaseType) -> Self {
        Self {
            container_id,
            database_type,
            timestamp: chrono::Utc::now().timestamp(),
            active_connections: 0,
            max_connections: None,
            connections_by_state: HashMap::new(),
            queries_per_second: 0.0,
            query_breakdown: QueryBreakdown::default(),
            transactions_per_second: 0.0,
            commits_per_second: 0.0,
            rollbacks_per_second: 0.0,
            cache_hit_ratio: None,
            replication_lag_bytes: None,
            replication_status: None,
            extras: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryBreakdown {
    pub select_count: u64,
    pub insert_count: u64,
    pub update_count: u64,
    pub delete_count: u64,
}

impl QueryBreakdown {
    pub fn total(&self) -> u64 {
        self.select_count + self.insert_count + self.update_count + self.delete_count
    }
}
