use crate::container::DatabaseType;
use crate::error::Result;

use super::collector::DockerDatabaseMetricsCollector;
use super::models::{DatabaseMetrics, QueryBreakdown};

/// Collect MySQL-specific metrics using mysql CLI via Docker exec
pub async fn collect_metrics(
    collector: &DockerDatabaseMetricsCollector,
    container_id: &str,
) -> Result<DatabaseMetrics> {
    let mut metrics = DatabaseMetrics::new(container_id.to_string(), DatabaseType::MySQL);

    // Get previous sample for rate calculation
    let previous = collector.get_previous_sample(container_id).await;
    let time_delta = if let Some(ref prev) = previous {
        (metrics.timestamp - prev.timestamp) as f64
    } else {
        1.0
    };

    // Query 1: Connection stats
    if let Ok(conn_output) = collector
        .exec_query(
            container_id,
            vec![
                "mysql",
                "-u",
                "root",
                "-pmysql",
                "-N",
                "-e",
                "SHOW STATUS WHERE Variable_name IN ('Threads_connected', 'Max_used_connections');",
            ],
        )
        .await
    {
        for line in conn_output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                match parts[0] {
                    "Threads_connected" => {
                        if let Ok(count) = parts[1].parse::<u64>() {
                            metrics.active_connections = count;
                        }
                    }
                    "Max_used_connections" => {
                        // Store for reference
                        if let Ok(max_used) = parts[1].parse::<u64>() {
                            metrics
                                .extras
                                .insert("max_used_connections".to_string(), max_used.into());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Query 2: Max connections setting
    if let Ok(max_conn_output) = collector
        .exec_query(
            container_id,
            vec![
                "mysql",
                "-u",
                "root",
                "-pmysql",
                "-N",
                "-e",
                "SHOW VARIABLES WHERE Variable_name = 'max_connections';",
            ],
        )
        .await
    {
        let parts: Vec<&str> = max_conn_output.trim().split_whitespace().collect();
        if parts.len() == 2 {
            if let Ok(max_conn) = parts[1].parse::<u64>() {
                metrics.max_connections = Some(max_conn);
            }
        }
    }

    // Query 3: Query counters
    if let Ok(query_output) = collector
        .exec_query(
            container_id,
            vec![
                "mysql",
                "-u",
                "root",
                "-pmysql",
                "-N",
                "-e",
                "SHOW GLOBAL STATUS WHERE Variable_name IN ('Com_select', 'Com_insert', 'Com_update', 'Com_delete', 'Com_commit', 'Com_rollback');",
            ],
        )
        .await
    {
        let mut select_count = 0u64;
        let mut insert_count = 0u64;
        let mut update_count = 0u64;
        let mut delete_count = 0u64;
        let mut commits = 0u64;
        let mut rollbacks = 0u64;

        for line in query_output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                let value = parts[1].parse::<u64>().unwrap_or(0);
                match parts[0] {
                    "Com_select" => select_count = value,
                    "Com_insert" => insert_count = value,
                    "Com_update" => update_count = value,
                    "Com_delete" => delete_count = value,
                    "Com_commit" => commits = value,
                    "Com_rollback" => rollbacks = value,
                    _ => {}
                }
            }
        }

        // Calculate rates if we have previous sample
        if let Some(ref prev) = previous {
            let select_delta = select_count.saturating_sub(
                prev.extras
                    .get("cumulative_select")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            );
            let insert_delta = insert_count.saturating_sub(
                prev.extras
                    .get("cumulative_insert")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            );
            let update_delta = update_count.saturating_sub(
                prev.extras
                    .get("cumulative_update")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            );
            let delete_delta = delete_count.saturating_sub(
                prev.extras
                    .get("cumulative_delete")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            );

            metrics.query_breakdown = QueryBreakdown {
                select_count: select_delta,
                insert_count: insert_delta,
                update_count: update_delta,
                delete_count: delete_delta,
            };

            metrics.queries_per_second = metrics.query_breakdown.total() as f64 / time_delta;

            let commits_delta = commits.saturating_sub(
                prev.extras
                    .get("cumulative_commits")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            );
            let rollbacks_delta = rollbacks.saturating_sub(
                prev.extras
                    .get("cumulative_rollbacks")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
            );

            metrics.commits_per_second = commits_delta as f64 / time_delta;
            metrics.rollbacks_per_second = rollbacks_delta as f64 / time_delta;
            metrics.transactions_per_second =
                (commits_delta + rollbacks_delta) as f64 / time_delta;
        }

        // Store cumulative values
        metrics
            .extras
            .insert("cumulative_select".to_string(), select_count.into());
        metrics
            .extras
            .insert("cumulative_insert".to_string(), insert_count.into());
        metrics
            .extras
            .insert("cumulative_update".to_string(), update_count.into());
        metrics
            .extras
            .insert("cumulative_delete".to_string(), delete_count.into());
        metrics
            .extras
            .insert("cumulative_commits".to_string(), commits.into());
        metrics
            .extras
            .insert("cumulative_rollbacks".to_string(), rollbacks.into());
    }

    // Query 4: InnoDB buffer pool hit ratio
    if let Ok(buffer_output) = collector
        .exec_query(
            container_id,
            vec![
                "mysql",
                "-u",
                "root",
                "-pmysql",
                "-N",
                "-e",
                "SHOW GLOBAL STATUS WHERE Variable_name IN ('Innodb_buffer_pool_read_requests', 'Innodb_buffer_pool_reads');",
            ],
        )
        .await
    {
        let mut read_requests = 0u64;
        let mut disk_reads = 0u64;

        for line in buffer_output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                let value = parts[1].parse::<u64>().unwrap_or(0);
                match parts[0] {
                    "Innodb_buffer_pool_read_requests" => read_requests = value,
                    "Innodb_buffer_pool_reads" => disk_reads = value,
                    _ => {}
                }
            }
        }

        if read_requests > 0 {
            let cache_hits = read_requests.saturating_sub(disk_reads);
            let hit_ratio = (cache_hits as f64 / read_requests as f64) * 100.0;
            metrics.cache_hit_ratio = Some(hit_ratio);
        }
    }

    // Query 5: Replication status (optional, may not be configured)
    if let Ok(repl_output) = collector
        .exec_query(
            container_id,
            vec!["mysql", "-u", "root", "-pmysql", "-N", "-e", "SHOW SLAVE STATUS\\G"],
        )
        .await
    {
        if !repl_output.is_empty() && !repl_output.contains("Empty set") {
            // Parse replication information
            for line in repl_output.lines() {
                if line.contains("Seconds_Behind_Master:") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() == 2 {
                        if let Ok(seconds) = parts[1].trim().parse::<u64>() {
                            // Convert seconds to approximate bytes (this is a rough estimate)
                            metrics.replication_lag_bytes = Some(seconds * 1024);
                            metrics.replication_status = Some("Replicating".to_string());
                        }
                    }
                } else if line.contains("Slave_IO_Running:") || line.contains("Slave_SQL_Running:") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() == 2 && parts[1].trim() == "Yes" {
                        if metrics.replication_status.is_none() {
                            metrics.replication_status = Some("Active".to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(metrics)
}
