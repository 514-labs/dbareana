use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::error::Result;
use super::metrics::ContainerMetrics;

/// Trait for collecting container metrics
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    /// Collect a single metrics snapshot for a container
    async fn collect(&self, container_id: &str) -> Result<ContainerMetrics>;

    /// Stream metrics for a container with periodic updates
    /// Returns a stream that yields metrics at regular intervals
    async fn stream(&self, container_id: &str) -> BoxStream<'_, Result<ContainerMetrics>>;

    /// Collect metrics for all running containers
    async fn collect_all(&self) -> Result<Vec<ContainerMetrics>>;
}
