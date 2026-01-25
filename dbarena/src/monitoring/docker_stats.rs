use async_trait::async_trait;
use bollard::Docker;
use bollard::container::StatsOptions;
use futures::{StreamExt, stream::{self, BoxStream}};
use chrono::Utc;
use std::sync::Arc;

use crate::error::{DBArenaError, Result};
use super::collector::MetricsCollector;
use super::metrics::{ContainerMetrics, CpuMetrics, MemoryMetrics, NetworkMetrics, BlockIoMetrics};

/// Docker stats-based metrics collector
pub struct DockerStatsCollector {
    docker: Arc<Docker>,
}

impl DockerStatsCollector {
    pub fn new(docker: Arc<Docker>) -> Self {
        Self { docker }
    }

    /// Calculate CPU percentage from Docker stats
    fn calculate_cpu_percent(
        cpu_delta: f64,
        system_delta: f64,
        num_cpus: u64,
    ) -> f64 {
        if system_delta > 0.0 && cpu_delta > 0.0 {
            (cpu_delta / system_delta) * num_cpus as f64 * 100.0
        } else {
            0.0
        }
    }
}

#[async_trait]
impl MetricsCollector for DockerStatsCollector {
    async fn collect(&self, container_id: &str) -> Result<ContainerMetrics> {
        // Get container info for name
        let inspect = self.docker
            .inspect_container(container_id, None)
            .await
            .map_err(|_| DBArenaError::ContainerNotFound(container_id.to_string()))?;

        let container_name = inspect.name
            .unwrap_or_else(|| container_id.to_string())
            .trim_start_matches('/')
            .to_string();

        // Get stats (one-shot mode)
        let stats_options = StatsOptions {
            stream: false,
            one_shot: true,
        };

        let mut stats_stream = self.docker.stats(container_id, Some(stats_options));

        let stats = stats_stream
            .next()
            .await
            .ok_or_else(|| DBArenaError::MonitoringError("No stats available".to_string()))?
            .map_err(|e| DBArenaError::MonitoringError(format!("Failed to get stats: {}", e)))?;

        // Parse CPU metrics
        let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
            - stats.precpu_stats.cpu_usage.total_usage as f64;
        let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
            - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
        let num_cpus = stats.cpu_stats.online_cpus.unwrap_or(
            stats.cpu_stats.cpu_usage.percpu_usage
                .as_ref()
                .map(|v| v.len() as u64)
                .unwrap_or(1)
        );

        let cpu_percent = Self::calculate_cpu_percent(cpu_delta, system_delta, num_cpus);

        // Parse memory metrics
        let memory_usage = stats.memory_stats.usage.unwrap_or(0);
        let memory_limit = stats.memory_stats.limit.unwrap_or(memory_usage);
        let memory_percent = if memory_limit > 0 {
            (memory_usage as f64 / memory_limit as f64) * 100.0
        } else {
            0.0
        };

        // Parse network metrics
        let (rx_bytes, tx_bytes) = stats.networks
            .as_ref()
            .map(|networks| {
                networks.values().fold((0u64, 0u64), |(rx, tx), net| {
                    (rx + net.rx_bytes, tx + net.tx_bytes)
                })
            })
            .unwrap_or((0, 0));

        // Parse block I/O metrics
        let (read_bytes, write_bytes) = stats.blkio_stats.io_service_bytes_recursive
            .as_ref()
            .map(|io_stats| {
                io_stats.iter().fold((0u64, 0u64), |(read, write), stat| {
                    match stat.op.as_str() {
                        "read" | "Read" => (read + stat.value, write),
                        "write" | "Write" => (read, write + stat.value),
                        _ => (read, write),
                    }
                })
            })
            .unwrap_or((0, 0));

        Ok(ContainerMetrics {
            container_id: container_id.to_string(),
            container_name,
            timestamp: Utc::now().timestamp(),
            cpu: CpuMetrics {
                usage_percent: cpu_percent,
                num_cores: num_cpus,
            },
            memory: MemoryMetrics {
                usage: memory_usage,
                limit: memory_limit,
                percent: memory_percent,
            },
            network: NetworkMetrics {
                rx_bytes,
                tx_bytes,
                rx_rate: 0.0,
                tx_rate: 0.0,
            },
            block_io: BlockIoMetrics {
                read_bytes,
                write_bytes,
                read_rate: 0.0,
                write_rate: 0.0,
            },
        })
    }

    async fn stream(&self, container_id: &str) -> BoxStream<'_, Result<ContainerMetrics>> {
        let docker = self.docker.clone();
        let container_id = container_id.to_string();

        let stream = stream::unfold(
            (docker, container_id, None),
            move |(docker, container_id, previous_metrics)| async move {
                let collector = DockerStatsCollector::new(docker.clone());

                match collector.collect(&container_id).await {
                    Ok(mut metrics) => {
                        // Calculate rates if we have previous metrics
                        if let Some(prev) = previous_metrics {
                            metrics.calculate_rates(&prev);
                        }

                        let prev = Some(metrics.clone());
                        Some((Ok(metrics), (docker, container_id, prev)))
                    }
                    Err(e) => Some((Err(e), (docker, container_id, previous_metrics))),
                }
            },
        );

        Box::pin(stream)
    }

    async fn collect_all(&self) -> Result<Vec<ContainerMetrics>> {
        // List all running containers
        use bollard::container::ListContainersOptions;
        use std::collections::HashMap;

        let mut filters = HashMap::new();
        filters.insert("status", vec!["running"]);

        let options = Some(ListContainersOptions {
            filters,
            ..Default::default()
        });

        let containers = self.docker
            .list_containers(options)
            .await
            .map_err(|e| DBArenaError::Other(format!("Failed to list containers: {}", e)))?;

        let mut metrics = Vec::new();
        for container in containers {
            if let Some(id) = container.id {
                match self.collect(&id).await {
                    Ok(m) => metrics.push(m),
                    Err(e) => {
                        tracing::warn!("Failed to collect metrics for {}: {}", id, e);
                    }
                }
            }
        }

        Ok(metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_cpu_percent() {
        // Test normal case
        let cpu_percent = DockerStatsCollector::calculate_cpu_percent(
            50000000.0,  // 50ms CPU time
            100000000.0, // 100ms system time
            2,           // 2 CPUs
        );
        assert_eq!(cpu_percent, 100.0); // (50/100) * 2 * 100 = 100%

        // Test zero delta
        let cpu_percent = DockerStatsCollector::calculate_cpu_percent(0.0, 0.0, 2);
        assert_eq!(cpu_percent, 0.0);

        // Test single CPU
        let cpu_percent = DockerStatsCollector::calculate_cpu_percent(
            25000000.0,  // 25ms CPU time
            100000000.0, // 100ms system time
            1,           // 1 CPU
        );
        assert_eq!(cpu_percent, 25.0);
    }
}
