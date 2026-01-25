use crate::container::DatabaseType;
use crate::error::Result;

use super::collector::DockerDatabaseMetricsCollector;
use super::models::{DatabaseMetrics, QueryBreakdown};

/// Collect PostgreSQL-specific metrics using psql via Docker exec
pub async fn collect_metrics(
    collector: &DockerDatabaseMetricsCollector,
    container_id: &str,
) -> Result<DatabaseMetrics> {
    let mut metrics = DatabaseMetrics::new(container_id.to_string(), DatabaseType::Postgres);

    // Get previous sample for rate calculation
    let previous = collector.get_previous_sample(container_id).await;
    let time_delta = if let Some(ref prev) = previous {
        (metrics.timestamp - prev.timestamp) as f64
    } else {
        1.0 // Default to 1 second if no previous sample
    };

    // Query 1: Connection states
    if let Ok(conn_output) = collector
        .exec_query(
            container_id,
            vec![
                "psql",
                "-U",
                "postgres",
                "-d",
                "postgres",
                "-t",
                "-A",
                "-F",
                ",",
                "-c",
                "SELECT COALESCE(state, 'unknown'), COUNT(*) FROM pg_stat_activity WHERE datname = current_database() GROUP BY state;",
            ],
        )
        .await
    {
        for line in conn_output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                let state = parts[0].to_string();
                if let Ok(count) = parts[1].parse::<u64>() {
                    metrics.connections_by_state.insert(state.clone(), count);
                    if state == "active" || state == "idle" || state == "idle in transaction" {
                        metrics.active_connections += count;
                    }
                }
            }
        }
    }

    // Query 2: Max connections
    if let Ok(max_conn_output) = collector
        .exec_query(
            container_id,
            vec![
                "psql",
                "-U",
                "postgres",
                "-d",
                "postgres",
                "-t",
                "-A",
                "-c",
                "SHOW max_connections;",
            ],
        )
        .await
    {
        if let Ok(max_conn) = max_conn_output.trim().parse::<u64>() {
            metrics.max_connections = Some(max_conn);
        }
    }

    // Query 3: Database statistics (commits, rollbacks, queries)
    if let Ok(stats_output) = collector
        .exec_query(
            container_id,
            vec![
                "psql",
                "-U",
                "postgres",
                "-d",
                "postgres",
                "-t",
                "-A",
                "-F",
                ",",
                "-c",
                "SELECT numbackends, xact_commit, xact_rollback, tup_returned, tup_fetched, tup_inserted, tup_updated, tup_deleted FROM pg_stat_database WHERE datname = current_database();",
            ],
        )
        .await
    {
        let line = stats_output.trim();
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 8 {
            // Parse cumulative counters
            let commits = parts[1].parse::<u64>().unwrap_or(0);
            let rollbacks = parts[2].parse::<u64>().unwrap_or(0);
            let rows_returned = parts[3].parse::<u64>().unwrap_or(0);
            let rows_fetched = parts[4].parse::<u64>().unwrap_or(0);
            let rows_inserted = parts[5].parse::<u64>().unwrap_or(0);
            let rows_updated = parts[6].parse::<u64>().unwrap_or(0);
            let rows_deleted = parts[7].parse::<u64>().unwrap_or(0);

            // Calculate rates if we have a previous sample
            if let Some(ref prev) = previous {
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

                // Estimate query breakdown from row operations
                // This is an approximation since PostgreSQL doesn't track query types directly
                let select_delta = rows_returned.saturating_sub(
                    prev.extras
                        .get("cumulative_rows_returned")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                );
                let insert_delta = rows_inserted.saturating_sub(
                    prev.extras
                        .get("cumulative_rows_inserted")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                );
                let update_delta = rows_updated.saturating_sub(
                    prev.extras
                        .get("cumulative_rows_updated")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                );
                let delete_delta = rows_deleted.saturating_sub(
                    prev.extras
                        .get("cumulative_rows_deleted")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                );

                metrics.query_breakdown = QueryBreakdown {
                    select_count: select_delta,
                    insert_count: insert_delta,
                    update_count: update_delta,
                    delete_count: delete_delta,
                };

                // Use transactions as a proxy for query activity
                // This is more accurate than row operations since background processes
                // don't inflate the count, and most user queries run in transactions
                // Note: In PostgreSQL, even single queries outside explicit transactions
                // count as implicit transactions (autocommit)
                let total_transactions = commits_delta + rollbacks_delta;
                metrics.queries_per_second = total_transactions as f64 / time_delta;
            }

            // Store cumulative values for next iteration
            metrics
                .extras
                .insert("cumulative_commits".to_string(), commits.into());
            metrics
                .extras
                .insert("cumulative_rollbacks".to_string(), rollbacks.into());
            metrics
                .extras
                .insert("cumulative_rows_returned".to_string(), rows_returned.into());
            metrics
                .extras
                .insert("cumulative_rows_fetched".to_string(), rows_fetched.into());
            metrics
                .extras
                .insert("cumulative_rows_inserted".to_string(), rows_inserted.into());
            metrics
                .extras
                .insert("cumulative_rows_updated".to_string(), rows_updated.into());
            metrics
                .extras
                .insert("cumulative_rows_deleted".to_string(), rows_deleted.into());
        }
    }

    // Query 4: Cache hit ratio
    if let Ok(cache_output) = collector
        .exec_query(
            container_id,
            vec![
                "psql",
                "-U",
                "postgres",
                "-d",
                "postgres",
                "-t",
                "-A",
                "-c",
                "SELECT COALESCE(ROUND(sum(heap_blks_hit) / NULLIF(sum(heap_blks_hit + heap_blks_read), 0) * 100, 2), 0) FROM pg_statio_user_tables;",
            ],
        )
        .await
    {
        if let Ok(ratio) = cache_output.trim().parse::<f64>() {
            metrics.cache_hit_ratio = Some(ratio);
        }
    }

    // Query 5: Replication lag (if replication is set up)
    if let Ok(repl_output) = collector
        .exec_query(
            container_id,
            vec![
                "psql",
                "-U",
                "postgres",
                "-d",
                "postgres",
                "-t",
                "-A",
                "-F",
                ",",
                "-c",
                "SELECT slot_name, pg_wal_lsn_diff(pg_current_wal_lsn(), confirmed_flush_lsn) FROM pg_replication_slots WHERE slot_type = 'logical';",
            ],
        )
        .await
    {
        for line in repl_output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                if let Ok(lag_bytes) = parts[1].parse::<u64>() {
                    metrics.replication_lag_bytes = Some(lag_bytes);
                    metrics.replication_status = Some(format!("Active ({})", parts[0]));
                    break;
                }
            }
        }
    }

    Ok(metrics)
}
