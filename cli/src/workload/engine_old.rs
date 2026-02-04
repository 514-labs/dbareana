use anyhow::{anyhow, Result};
use bollard::Docker;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::container::DatabaseType;
use crate::database_metrics::collector::DockerDatabaseMetricsCollector;
use crate::workload::config::{OperationWeights, WorkloadConfig};
use crate::workload::rate_limiter::RateLimiter;
use crate::workload::stats::{MetricSample, WorkloadStats};

/// Main workload generation engine
pub struct WorkloadEngine {
    container_id: String,
    db_type: DatabaseType,
    config: WorkloadConfig,
    docker_client: Arc<Docker>,
    rate_limiter: Arc<RateLimiter>,
    stats: Arc<WorkloadStats>,
}

impl WorkloadEngine {
    pub fn new(
        container_id: String,
        db_type: DatabaseType,
        config: WorkloadConfig,
        docker_client: Arc<Docker>,
    ) -> Self {
        let rate_limiter = Arc::new(RateLimiter::new(config.target_tps));
        let stats = Arc::new(WorkloadStats::new());

        Self {
            container_id,
            db_type,
            config,
            docker_client,
            rate_limiter,
            stats,
        }
    }

    /// Run the workload
    pub async fn run(&self) -> Result<WorkloadStats> {
        println!("Starting workload with {} workers", self.config.connections);
        println!("Target TPS: {}", self.config.target_tps);

        // Create channel for metric samples
        let (tx, mut rx) = mpsc::channel::<MetricSample>(1000);

        // Spawn worker tasks
        let mut worker_handles = vec![];
        for worker_id in 0..self.config.connections {
            let handle = self.spawn_worker(worker_id, tx.clone()).await;
            worker_handles.push(handle);
        }

        // Drop the original sender so the channel closes when workers are done
        drop(tx);

        // Spawn aggregator task
        let stats = self.stats.clone();
        let aggregator = tokio::spawn(async move {
            while let Some(sample) = rx.recv().await {
                stats.record_sample(sample);
            }
        });

        // Wait for duration or transaction count
        if let Some(duration) = self.config.duration_seconds {
            println!("Running for {} seconds", duration);
            sleep(Duration::from_secs(duration)).await;
        } else if let Some(count) = self.config.transaction_count {
            println!("Running until {} transactions", count);
            while self.stats.total() < count {
                sleep(Duration::from_millis(100)).await;
            }
        }

        // Signal workers to stop (they'll check elapsed time or transaction count)
        for handle in worker_handles {
            handle.abort();
        }

        // Wait for aggregator to finish processing
        let _ = aggregator.await;

        // Return final stats (clone the Arc contents)
        Ok(WorkloadStats {
            total_transactions: std::sync::atomic::AtomicU64::new(self.stats.total()),
            successful_transactions: std::sync::atomic::AtomicU64::new(self.stats.success_count()),
            failed_transactions: std::sync::atomic::AtomicU64::new(self.stats.failure_count()),
            latency_histogram: std::sync::Mutex::new(
                self.stats.latency_histogram.lock().unwrap().clone(),
            ),
            errors: std::sync::Mutex::new(self.stats.errors.lock().unwrap().clone()),
            operation_counts: std::sync::Mutex::new(
                self.stats.operation_counts.lock().unwrap().clone(),
            ),
            start_time: self.stats.start_time,
        })
    }

    /// Spawn a worker task
    async fn spawn_worker(
        &self,
        worker_id: usize,
        tx: mpsc::Sender<MetricSample>,
    ) -> tokio::task::JoinHandle<()> {
        let container_id = self.container_id.clone();
        let db_type = self.db_type;
        let config = self.config.clone();
        let docker_client = self.docker_client.clone();
        let rate_limiter = self.rate_limiter.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            let collector = DockerDatabaseMetricsCollector::new(docker_client);
            let mut rng = ChaCha8Rng::seed_from_u64(worker_id as u64);

            // Get operation weights
            let weights = if let Some(pattern) = config.pattern {
                pattern.operation_weights()
            } else if let Some(custom) = &config.custom_operations {
                custom.to_weights()
            } else {
                OperationWeights {
                    select: 0.5,
                    insert: 0.25,
                    update: 0.2,
                    delete: 0.05,
                }
            };

            loop {
                // Check termination conditions
                if let Some(duration) = config.duration_seconds {
                    if stats.elapsed() >= Duration::from_secs(duration) {
                        break;
                    }
                }

                if let Some(count) = config.transaction_count {
                    if stats.total() >= count {
                        break;
                    }
                }

                // Wait for rate limiter
                rate_limiter.acquire().await;

                // Select operation type based on weights
                let op_type = select_operation(&weights, &mut rng);

                // Execute operation
                let start = Instant::now();
                let result = execute_operation(
                    &collector,
                    &container_id,
                    db_type,
                    &op_type,
                    &config.tables,
                    &mut rng,
                )
                .await;

                let latency = start.elapsed();

                // Create metric sample
                let sample = match result {
                    Ok(_) => MetricSample {
                        worker_id,
                        operation_type: op_type.to_string(),
                        success: true,
                        latency_us: latency.as_micros() as u64,
                        error: None,
                    },
                    Err(e) => MetricSample {
                        worker_id,
                        operation_type: op_type.to_string(),
                        success: false,
                        latency_us: latency.as_micros() as u64,
                        error: Some(e.to_string()),
                    },
                };

                // Send to aggregator (ignore if channel is full)
                let _ = tx.send(sample).await;
            }
        })
    }

    /// Get a reference to the stats
    pub fn stats(&self) -> &Arc<WorkloadStats> {
        &self.stats
    }
}

/// Select an operation type based on weights
fn select_operation(weights: &OperationWeights, rng: &mut ChaCha8Rng) -> &'static str {
    let roll = rng.gen::<f64>();

    if roll < weights.select {
        "SELECT"
    } else if roll < weights.select + weights.insert {
        "INSERT"
    } else if roll < weights.select + weights.insert + weights.update {
        "UPDATE"
    } else {
        "DELETE"
    }
}

/// Execute a database operation
async fn execute_operation(
    collector: &DockerDatabaseMetricsCollector,
    container_id: &str,
    db_type: DatabaseType,
    op_type: &str,
    tables: &[String],
    rng: &mut ChaCha8Rng,
) -> Result<()> {
    // Select random table
    if tables.is_empty() {
        return Err(anyhow!("No tables configured for workload"));
    }
    let table_idx = rng.gen_range(0..tables.len());
    let table = &tables[table_idx];

    // Generate simple SQL (will be enhanced in Phase 6)
    let sql = match op_type {
        "SELECT" => format!("SELECT * FROM {} LIMIT 10", table),
        "INSERT" => format!("INSERT INTO {} DEFAULT VALUES", table),
        "UPDATE" => format!("UPDATE {} SET id = id WHERE id = 1", table),
        "DELETE" => format!("DELETE FROM {} WHERE 1=0", table), // No-op for now
        _ => return Err(anyhow!("Unknown operation type: {}", op_type)),
    };

    // Execute SQL via Docker exec
    let command = match db_type {
        DatabaseType::Postgres => {
            vec!["psql", "-U", "postgres", "-c", &sql]
        }
        DatabaseType::MySQL => {
            vec!["mysql", "-uroot", "-proot", "-e", &sql]
        }
        DatabaseType::SQLServer => {
            vec![
                "/opt/mssql-tools18/bin/sqlcmd",
                "-S",
                "localhost",
                "-U",
                "sa",
                "-P",
                "YourStrong@Passw0rd",
                "-C",
                "-Q",
                &sql,
            ]
        }
    };

    let output = collector.exec_query(container_id, command).await?;

    // Check for errors in output
    if output.to_lowercase().contains("error") {
        return Err(anyhow!("SQL execution error: {}", output));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workload::config::WorkloadPattern;

    #[test]
    fn test_select_operation() {
        let weights = WorkloadPattern::Balanced.operation_weights();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Generate 1000 operations and check distribution
        let mut counts = std::collections::HashMap::new();
        for _ in 0..1000 {
            let op = select_operation(&weights, &mut rng);
            *counts.entry(op).or_insert(0) += 1;
        }

        // Balanced pattern should have roughly 50% SELECT
        let select_count = counts.get("SELECT").unwrap_or(&0);
        assert!(*select_count > 400 && *select_count < 600);

        // Should have all operation types
        assert!(counts.contains_key("SELECT"));
        assert!(counts.contains_key("INSERT"));
        assert!(counts.contains_key("UPDATE"));
        assert!(counts.contains_key("DELETE"));
    }

    #[tokio::test]
    async fn test_workload_engine_creation() {
        let docker = Arc::new(Docker::connect_with_local_defaults().unwrap());

        let config = WorkloadConfig {
            name: "Test".to_string(),
            pattern: Some(WorkloadPattern::Balanced),
            custom_operations: None,
            custom_queries: None,
            tables: vec!["users".to_string()],
            connections: 10,
            target_tps: 100,
            duration_seconds: Some(1),
            transaction_count: None,
        };

        let engine = WorkloadEngine::new(
            "test-container".to_string(),
            DatabaseType::Postgres,
            config,
            docker,
        );

        assert_eq!(engine.config.connections, 10);
        assert_eq!(engine.config.target_tps, 100);
    }

    #[test]
    fn test_operation_weights_distribution() {
        let patterns = vec![
            WorkloadPattern::Oltp,
            WorkloadPattern::ReadHeavy,
            WorkloadPattern::WriteHeavy,
        ];

        for pattern in patterns {
            let weights = pattern.operation_weights();
            let mut rng = ChaCha8Rng::seed_from_u64(42);

            let mut counts = std::collections::HashMap::new();
            for _ in 0..10000 {
                let op = select_operation(&weights, &mut rng);
                *counts.entry(op).or_insert(0) += 1;
            }

            // Verify distributions match expected weights (within 5% tolerance)
            let select_ratio = *counts.get("SELECT").unwrap_or(&0) as f64 / 10000.0;
            assert!((select_ratio - weights.select).abs() < 0.05);
        }
    }
}
