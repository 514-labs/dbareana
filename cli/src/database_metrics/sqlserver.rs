use crate::container::DatabaseType;
use crate::error::Result;

use super::collector::DockerDatabaseMetricsCollector;
use super::models::{DatabaseMetrics, QueryBreakdown};

/// Collect SQL Server-specific metrics using sqlcmd via Docker exec
pub async fn collect_metrics(
    collector: &DockerDatabaseMetricsCollector,
    container_id: &str,
) -> Result<DatabaseMetrics> {
    let mut metrics = DatabaseMetrics::new(container_id.to_string(), DatabaseType::SQLServer);

    // Get previous sample for rate calculation
    let previous = collector.get_previous_sample(container_id).await;
    let time_delta = if let Some(ref prev) = previous {
        (metrics.timestamp - prev.timestamp) as f64
    } else {
        1.0
    };

    // SQL Server uses sa/YourStrong@Passw0rd as default credentials
    let sqlcmd_base = vec![
        "/opt/mssql-tools/bin/sqlcmd",
        "-S",
        "localhost",
        "-U",
        "sa",
        "-P",
        "YourStrong@Passw0rd",
        "-h",
        "-1",
        "-W",
        "-s",
        ",",
    ];

    // Query 1: Active connections
    let mut query_cmd = sqlcmd_base.clone();
    query_cmd.extend_from_slice(&[
        "-Q",
        "SELECT COUNT(*) as active_connections FROM sys.dm_exec_sessions WHERE is_user_process = 1;",
    ]);

    if let Ok(conn_output) = collector.exec_query(container_id, query_cmd).await {
        let line = conn_output.trim();
        if let Ok(count) = line.parse::<u64>() {
            metrics.active_connections = count;
        }
    }

    // Query 2: Max connections (server configuration)
    let mut query_cmd = sqlcmd_base.clone();
    query_cmd.extend_from_slice(&[
        "-Q",
        "SELECT CAST(value_in_use AS VARCHAR) FROM sys.configurations WHERE name = 'user connections';",
    ]);

    if let Ok(max_conn_output) = collector.exec_query(container_id, query_cmd).await {
        let line = max_conn_output.trim();
        if let Ok(max_conn) = line.parse::<u64>() {
            if max_conn > 0 {
                metrics.max_connections = Some(max_conn);
            }
        }
    }

    // Query 3: Performance counters (batch requests, transactions)
    let mut query_cmd = sqlcmd_base.clone();
    query_cmd.extend_from_slice(&[
        "-Q",
        "SELECT counter_name, cntr_value FROM sys.dm_os_performance_counters WHERE counter_name IN ('Batch Requests/sec', 'Transactions/sec') AND (instance_name = '_Total' OR instance_name = '');",
    ]);

    if let Ok(perf_output) = collector.exec_query(container_id, query_cmd).await {
        let mut batch_requests = 0u64;
        let mut transactions = 0u64;

        for line in perf_output.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let counter_name = parts[0].trim();
                if let Ok(value) = parts[1].trim().parse::<u64>() {
                    match counter_name {
                        "Batch Requests/sec" => batch_requests = value,
                        "Transactions/sec" => transactions = value,
                        _ => {}
                    }
                }
            }
        }

        // Calculate rates if we have previous sample
        if let Some(ref prev) = previous {
            let batch_delta = batch_requests.saturating_sub(
                prev.extras
                    .get("cumulative_batch_requests")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            );
            let trans_delta = transactions.saturating_sub(
                prev.extras
                    .get("cumulative_transactions")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            );

            // Use batch requests as a proxy for QPS
            metrics.queries_per_second = batch_delta as f64 / time_delta;
            metrics.transactions_per_second = trans_delta as f64 / time_delta;

            // SQL Server doesn't break down by query type easily, so we estimate
            // Assume most batches are queries, with some being writes
            let estimated_selects = (batch_delta as f64 * 0.7) as u64;
            let estimated_writes = batch_delta - estimated_selects;

            metrics.query_breakdown = QueryBreakdown {
                select_count: estimated_selects,
                insert_count: estimated_writes / 3,
                update_count: estimated_writes / 3,
                delete_count: estimated_writes / 3,
            };
        }

        // Store cumulative values
        metrics
            .extras
            .insert("cumulative_batch_requests".to_string(), batch_requests.into());
        metrics
            .extras
            .insert("cumulative_transactions".to_string(), transactions.into());
    }

    // Query 4: Page life expectancy (buffer pool efficiency)
    let mut query_cmd = sqlcmd_base.clone();
    query_cmd.extend_from_slice(&[
        "-Q",
        "SELECT cntr_value FROM sys.dm_os_performance_counters WHERE counter_name = 'Page life expectancy' AND object_name LIKE '%Buffer Manager%';",
    ]);

    if let Ok(ple_output) = collector.exec_query(container_id, query_cmd).await {
        let line = ple_output.trim();
        if let Ok(ple_seconds) = line.parse::<u64>() {
            // Page Life Expectancy in seconds - higher is better
            // Typical good values are > 300 seconds
            // Convert to a cache hit ratio approximation: higher PLE = better cache
            // This is a rough estimate: 300+ seconds = 95%, 100-300 = 80%, < 100 = 60%
            let estimated_hit_ratio = if ple_seconds >= 300 {
                95.0 + ((ple_seconds - 300) as f64 / 100.0).min(5.0)
            } else if ple_seconds >= 100 {
                80.0 + ((ple_seconds - 100) as f64 / 200.0) * 15.0
            } else {
                60.0 + (ple_seconds as f64 / 100.0) * 20.0
            };

            metrics.cache_hit_ratio = Some(estimated_hit_ratio);
            metrics
                .extras
                .insert("page_life_expectancy_seconds".to_string(), ple_seconds.into());
        }
    }

    // Query 5: CDC capture latency (if CDC is enabled)
    let mut query_cmd = sqlcmd_base.clone();
    query_cmd.extend_from_slice(&[
        "-Q",
        "SELECT SUM(DATEDIFF(SECOND, last_commit_time, GETDATE())) as total_lag FROM cdc.lsn_time_mapping WHERE tran_begin_time IS NOT NULL AND last_commit_time > DATEADD(MINUTE, -5, GETDATE());",
    ]);

    if let Ok(cdc_output) = collector.exec_query(container_id, query_cmd).await {
        let line = cdc_output.trim();
        if !line.is_empty() && line != "NULL" {
            if let Ok(lag_seconds) = line.parse::<u64>() {
                // Convert seconds to approximate bytes (rough estimate)
                metrics.replication_lag_bytes = Some(lag_seconds * 1024);
                metrics.replication_status = Some("CDC Active".to_string());
            }
        }
    }

    Ok(metrics)
}
