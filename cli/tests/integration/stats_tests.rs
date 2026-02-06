use bollard::Docker;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use dbarena::monitoring::{DockerStatsCollector, MetricsCollector};
use dbarena::container::{ContainerManager, DockerClient, ContainerConfig, DatabaseType};

#[path = "../common/mod.rs"]
mod common;

/// Helper to create a test container
async fn create_test_container() -> Result<String, Box<dyn std::error::Error>> {
    let docker_client = DockerClient::new()?;
    let manager = ContainerManager::new(docker_client);

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name(common::unique_container_name("test-stats-container"))
        .with_version("16-alpine".to_string()); // Use alpine for smaller image

    let container = manager.create_container(config).await?;
    manager.start_container(&container.id).await?;

    // Wait a bit for container to stabilize
    sleep(Duration::from_secs(5)).await;

    Ok(container.id)
}

/// Helper to cleanup test container
async fn cleanup_container(container_id: &str) {
    if let Ok(docker) = Docker::connect_with_local_defaults() {
        let _ = docker.stop_container(container_id, None).await;
        let _ = docker.remove_container(
            container_id,
            Some(bollard::container::RemoveContainerOptions {
                force: true,
                v: true,
                ..Default::default()
            }),
        ).await;
    }
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_collect_metrics_from_running_container() {
    // Create test container
    let container_id = match create_test_container().await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to create test container: {}", e);
            return;
        }
    };

    // Create collector
    let docker = Docker::connect_with_local_defaults().unwrap();
    let collector = DockerStatsCollector::new(Arc::new(docker));

    // Collect metrics
    let result = collector.collect(&container_id).await;

    // Cleanup
    cleanup_container(&container_id).await;

    // Verify
    assert!(result.is_ok(), "Failed to collect metrics: {:?}", result.err());

    let metrics = result.unwrap();
    assert_eq!(metrics.container_id, container_id);
    assert!(!metrics.container_name.is_empty());
    assert!(metrics.cpu.usage_percent >= 0.0);
    assert!(metrics.cpu.usage_percent <= 100.0 * metrics.cpu.num_cores as f64);
    assert!(metrics.memory.usage > 0);
    assert!(metrics.memory.limit > 0);
    assert!(metrics.memory.percent >= 0.0);
    assert!(metrics.memory.percent <= 100.0);
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_collect_metrics_multiple_times() {
    let container_id = match create_test_container().await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to create test container: {}", e);
            return;
        }
    };

    let docker = Docker::connect_with_local_defaults().unwrap();
    let collector = DockerStatsCollector::new(Arc::new(docker));

    // Collect multiple metrics snapshots
    let mut previous = None;
    for i in 0..3 {
        sleep(Duration::from_secs(2)).await;

        let mut metrics = collector.collect(&container_id).await.unwrap();

        // Calculate rates if we have previous metrics
        if let Some(prev) = previous {
            metrics.calculate_rates(&prev);

            // After first iteration, we should have rate data
            if i > 0 {
                // Rates should be calculated (may be 0 if no I/O)
                assert!(metrics.network.rx_rate >= 0.0);
                assert!(metrics.network.tx_rate >= 0.0);
                assert!(metrics.block_io.read_rate >= 0.0);
                assert!(metrics.block_io.write_rate >= 0.0);
            }
        }

        previous = Some(metrics);
    }

    cleanup_container(&container_id).await;
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_collect_all_containers() {
    let container_id = match create_test_container().await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to create test container: {}", e);
            return;
        }
    };

    let docker = Docker::connect_with_local_defaults().unwrap();
    let collector = DockerStatsCollector::new(Arc::new(docker));

    let result = collector.collect_all().await;

    cleanup_container(&container_id).await;

    assert!(result.is_ok());
    let all_metrics = result.unwrap();

    // Should have at least our test container
    assert!(!all_metrics.is_empty());

    // Find our container in the results
    let found = all_metrics.iter().any(|m| m.container_id == container_id);
    assert!(found, "Test container not found in collect_all results");
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_metrics_stream() {
    let container_id = match create_test_container().await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to create test container: {}", e);
            return;
        }
    };

    let docker = Docker::connect_with_local_defaults().unwrap();
    let collector = DockerStatsCollector::new(Arc::new(docker));

    use futures::StreamExt;

    let mut stream = collector.stream(&container_id).await;

    // Collect a few metrics from the stream
    let mut count = 0;
    while let Some(result) = stream.next().await {
        if count >= 3 {
            break;
        }

        assert!(result.is_ok(), "Stream returned error: {:?}", result.err());
        let metrics = result.unwrap();
        assert_eq!(metrics.container_id, container_id);

        count += 1;
        sleep(Duration::from_secs(1)).await;
    }

    cleanup_container(&container_id).await;

    assert_eq!(count, 3, "Should have collected 3 metrics from stream");
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_collect_nonexistent_container() {
    let docker = Docker::connect_with_local_defaults().unwrap();
    let collector = DockerStatsCollector::new(Arc::new(docker));

    let result = collector.collect("nonexistent-container-id-12345").await;

    // Should return an error
    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_metrics_consistency() {
    let container_id = match create_test_container().await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to create test container: {}", e);
            return;
        }
    };

    let docker = Docker::connect_with_local_defaults().unwrap();
    let collector = DockerStatsCollector::new(Arc::new(docker));

    // Collect metrics twice in quick succession
    let metrics1 = collector.collect(&container_id).await.unwrap();
    sleep(Duration::from_millis(500)).await;
    let metrics2 = collector.collect(&container_id).await.unwrap();

    cleanup_container(&container_id).await;

    // Verify consistency
    assert_eq!(metrics1.container_id, metrics2.container_id);
    assert_eq!(metrics1.container_name, metrics2.container_name);

    // Memory limit should be the same
    assert_eq!(metrics1.memory.limit, metrics2.memory.limit);

    // CPU cores should be the same
    assert_eq!(metrics1.cpu.num_cores, metrics2.cpu.num_cores);

    // Cumulative counters should be monotonically increasing or equal
    assert!(metrics2.network.rx_bytes >= metrics1.network.rx_bytes);
    assert!(metrics2.network.tx_bytes >= metrics1.network.tx_bytes);
    assert!(metrics2.block_io.read_bytes >= metrics1.block_io.read_bytes);
    assert!(metrics2.block_io.write_bytes >= metrics1.block_io.write_bytes);
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_cpu_usage_bounds() {
    let container_id = match create_test_container().await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to create test container: {}", e);
            return;
        }
    };

    let docker = Docker::connect_with_local_defaults().unwrap();
    let collector = DockerStatsCollector::new(Arc::new(docker));

    // Collect multiple samples to verify CPU percentage is within bounds
    for _ in 0..5 {
        let metrics = collector.collect(&container_id).await.unwrap();

        // CPU usage should be non-negative
        assert!(metrics.cpu.usage_percent >= 0.0);

        // CPU usage per core should not exceed 100% * num_cores
        // Allow some margin for measurement variance
        let max_percent = 100.0 * metrics.cpu.num_cores as f64 * 1.1; // 10% margin
        assert!(
            metrics.cpu.usage_percent <= max_percent,
            "CPU usage {}% exceeds maximum {}% for {} cores",
            metrics.cpu.usage_percent,
            max_percent,
            metrics.cpu.num_cores
        );

        sleep(Duration::from_secs(1)).await;
    }

    cleanup_container(&container_id).await;
}

#[tokio::test]
#[ignore] // Requires Docker
async fn test_memory_percentage_calculation() {
    let container_id = match create_test_container().await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to create test container: {}", e);
            return;
        }
    };

    let docker = Docker::connect_with_local_defaults().unwrap();
    let collector = DockerStatsCollector::new(Arc::new(docker));

    let metrics = collector.collect(&container_id).await.unwrap();

    cleanup_container(&container_id).await;

    // Verify memory percentage matches usage/limit
    let calculated_percent = (metrics.memory.usage as f64 / metrics.memory.limit as f64) * 100.0;
    let diff = (metrics.memory.percent - calculated_percent).abs();

    assert!(
        diff < 1.0,
        "Memory percentage mismatch: reported {}%, calculated {}%",
        metrics.memory.percent,
        calculated_percent
    );

    // Memory percentage should be in valid range
    assert!(metrics.memory.percent >= 0.0);
    assert!(metrics.memory.percent <= 100.0);
}
