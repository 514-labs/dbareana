use dbarena::database_metrics::{DatabaseMetrics, QueryBreakdown};
use dbarena::container::DatabaseType;

#[test]
fn test_database_metrics_creation() {
    let metrics = DatabaseMetrics::new("test-container".to_string(), DatabaseType::Postgres);

    assert_eq!(metrics.container_id, "test-container");
    assert_eq!(metrics.database_type, DatabaseType::Postgres);
    assert_eq!(metrics.active_connections, 0);
    assert_eq!(metrics.queries_per_second, 0.0);
    assert_eq!(metrics.transactions_per_second, 0.0);
    assert!(metrics.max_connections.is_none());
    assert!(metrics.cache_hit_ratio.is_none());
}

#[test]
fn test_query_breakdown_total() {
    let breakdown = QueryBreakdown {
        select_count: 10,
        insert_count: 5,
        update_count: 3,
        delete_count: 2,
    };

    assert_eq!(breakdown.total(), 20);
}

#[test]
fn test_query_breakdown_default() {
    let breakdown = QueryBreakdown::default();

    assert_eq!(breakdown.select_count, 0);
    assert_eq!(breakdown.insert_count, 0);
    assert_eq!(breakdown.update_count, 0);
    assert_eq!(breakdown.delete_count, 0);
    assert_eq!(breakdown.total(), 0);
}

#[test]
fn test_database_metrics_serialization() {
    let metrics = DatabaseMetrics::new("test-container".to_string(), DatabaseType::MySQL);

    // Test that it can be serialized to JSON
    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("test-container"));
    // DatabaseType serialization might be lowercase or different
    assert!(json.contains("mysql") || json.contains("MySQL") || json.contains("Mysql"));

    // Test that it can be deserialized
    let deserialized: DatabaseMetrics = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.container_id, "test-container");
    assert_eq!(deserialized.database_type, DatabaseType::MySQL);
}
