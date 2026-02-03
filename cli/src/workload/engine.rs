// This file will replace engine.rs - creating as v2 first to avoid breaking existing code
use anyhow::{anyhow, Result};
use bollard::Docker;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex as TokioMutex};
use tokio::time::sleep;

use crate::container::DatabaseType;
use crate::database_metrics::collector::DockerDatabaseMetricsCollector;
use crate::workload::config::{OperationWeights, WorkloadConfig};
use crate::workload::metadata::{MetadataCollector, TableMetadata};
use crate::workload::operations::OperationGenerator;
use crate::workload::rate_limiter::RateLimiter;
use crate::workload::stats::{MetricSample, WorkloadStats};

/// Main workload generation engine with realistic operations
pub struct WorkloadEngine {
    container_id: String,
    db_type: DatabaseType,
    config: WorkloadConfig,
    docker_client: Arc<Docker>,
    rate_limiter: Arc<RateLimiter>,
    stats: Arc<WorkloadStats>,
    metadata: Arc<TokioMutex<HashMap<String, TableMetadata>>>,
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
            metadata: Arc::new(TokioMutex::new(HashMap::new())),
        }
    }

    /// Run the workload
    pub async fn run(&self) -> Result<WorkloadStats> {
        println!("Starting workload with {} workers", self.config.connections);
        println!("Target TPS: {}", self.config.target_tps);

        // Collect table metadata first
        println!("Collecting table metadata...");
        self.collect_metadata().await?;
        println!("Metadata collected for {} tables", self.config.tables.len());

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

        // Signal workers to stop
        for handle in worker_handles {
            handle.abort();
        }

        // Wait for aggregator to finish
        let _ = aggregator.await;

        // Return final stats
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

    /// Collect metadata for all tables
    async fn collect_metadata(&self) -> Result<()> {
        let collector = DockerDatabaseMetricsCollector::new(self.docker_client.clone());
        let mut metadata_collector = MetadataCollector::new(
            collector,
            self.container_id.clone(),
            self.db_type,
        );

        let mut metadata_map = self.metadata.lock().await;

        for table in &self.config.tables {
            match metadata_collector.get_metadata(table).await {
                Ok(metadata) => {
                    metadata_map.insert(table.clone(), metadata.clone());
                }
                Err(e) => {
                    println!("Warning: Failed to collect metadata for table '{}': {}", table, e);
                }
            }
        }

        if metadata_map.is_empty() {
            return Err(anyhow!("No table metadata could be collected"));
        }

        Ok(())
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
        let metadata = self.metadata.clone();

        tokio::spawn(async move {
            let collector = DockerDatabaseMetricsCollector::new(docker_client);
            let op_gen = OperationGenerator::new(db_type);
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

                // Select operation type
                let op_type = select_operation(&weights, &mut rng);

                // Execute operation
                let start = Instant::now();
                let result = execute_operation_with_metadata(
                    &collector,
                    &container_id,
                    db_type,
                    &op_gen,
                    op_type,
                    &metadata,
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

                // Send to aggregator
                let _ = tx.send(sample).await;
            }
        })
    }

    /// Get stats reference
    pub fn stats(&self) -> &Arc<WorkloadStats> {
        &self.stats
    }
}

/// Select operation type based on weights
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

/// Execute operation with realistic SQL
async fn execute_operation_with_metadata(
    collector: &DockerDatabaseMetricsCollector,
    container_id: &str,
    db_type: DatabaseType,
    op_gen: &OperationGenerator,
    op_type: &str,
    metadata_map: &Arc<TokioMutex<HashMap<String, TableMetadata>>>,
    tables: &[String],
    rng: &mut ChaCha8Rng,
) -> Result<()> {
    if tables.is_empty() {
        return Err(anyhow!("No tables configured"));
    }

    // Select random table
    let table_idx = rng.gen_range(0..tables.len());
    let table = &tables[table_idx];

    // Get metadata for table
    let metadata = {
        let map = metadata_map.lock().await;
        map.get(table).cloned()
    };

    let metadata = match metadata {
        Some(m) => m,
        None => {
            return Err(anyhow!("No metadata available for table: {}", table));
        }
    };

    // Generate operation using OperationGenerator
    let operation = match op_type {
        "SELECT" => op_gen.generate_select(&metadata, rng)?,
        "INSERT" => op_gen.generate_insert(&metadata, rng)?,
        "UPDATE" => op_gen.generate_update(&metadata, rng)?,
        "DELETE" => op_gen.generate_delete(&metadata, rng)?,
        _ => return Err(anyhow!("Unknown operation type: {}", op_type)),
    };

    let sql = operation.sql();

    // Execute SQL
    let command = match db_type {
        DatabaseType::Postgres => vec!["psql", "-U", "postgres", "-c", sql],
        DatabaseType::MySQL => vec!["mysql", "-uroot", "-proot", "-e", sql],
        DatabaseType::SQLServer => vec![
            "/opt/mssql-tools18/bin/sqlcmd",
            "-S", "localhost",
            "-U", "sa",
            "-P", "YourStrong@Passw0rd",
            "-C",
            "-Q", sql,
        ],
    };

    let output = collector.exec_query(container_id, command).await?;

    // Check for errors
    if output.to_lowercase().contains("error") && !output.contains("0 rows") {
        return Err(anyhow!("SQL error: {}", output));
    }

    Ok(())
}
