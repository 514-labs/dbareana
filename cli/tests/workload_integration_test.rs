use dbarena::container::DatabaseType;
use dbarena::workload::{WorkloadConfig, WorkloadEngine, WorkloadPattern};
use bollard::Docker;
use std::sync::Arc;
use std::time::Duration;

/// Integration test for workload engine
///
/// Note: This test demonstrates the workload engine functionality
/// but requires a running database container to execute against.
///
/// To run manually:
/// 1. Create a test container: `dbarena create postgres --name workload-test`
/// 2. Run this test: `cargo test workload_integration -- --ignored --nocapture`
#[tokio::test]
#[ignore] // Ignored by default since it requires a running container
async fn test_workload_engine_end_to_end() {
    // This test requires a running container named "workload-test"
    // Create one with: dbarena create postgres --name workload-test

    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => Arc::new(d),
        Err(_) => {
            println!("Docker not available - skipping integration test");
            return;
        }
    };

    // Create a minimal workload config
    let config = WorkloadConfig {
        name: "Integration Test Workload".to_string(),
        pattern: Some(WorkloadPattern::Balanced),
        custom_operations: None,
        custom_queries: None,
        tables: vec!["pg_database".to_string()], // Use system table that always exists
        connections: 5,
        target_tps: 50,
        duration_seconds: Some(2), // Short duration for testing
        transaction_count: None,
    };

    // Create workload engine
    let engine = WorkloadEngine::new(
        "workload-test".to_string(),
        DatabaseType::Postgres,
        config,
        docker,
    );

    println!("Starting workload engine test...");
    println!("Running 5 workers at 50 TPS for 2 seconds");

    // Run the workload
    let result = engine.run().await;

    match result {
        Ok(stats) => {
            println!("\n=== Workload Test Results ===");
            println!("Total transactions: {}", stats.total());
            println!("Successful: {}", stats.success_count());
            println!("Failed: {}", stats.failure_count());
            println!("Success rate: {:.2}%", stats.success_rate());
            println!("Duration: {:?}", stats.elapsed());
            println!("TPS: {:.2}", stats.tps());

            if let Some(p50) = stats.latency_percentile(0.50) {
                println!("P50 latency: {:.2}ms", p50 as f64 / 1000.0);
            }
            if let Some(p95) = stats.latency_percentile(0.95) {
                println!("P95 latency: {:.2}ms", p95 as f64 / 1000.0);
            }
            if let Some(p99) = stats.latency_percentile(0.99) {
                println!("P99 latency: {:.2}ms", p99 as f64 / 1000.0);
            }

            println!("\nOperation counts:");
            for (op, count) in stats.operation_counts() {
                println!("  {}: {}", op, count);
            }

            if !stats.error_counts().is_empty() {
                println!("\nErrors:");
                for (error, count) in stats.error_counts() {
                    println!("  {}: {}", error, count);
                }
            }

            // Verify basic expectations
            assert!(stats.total() > 0, "Should have executed some transactions");
            assert!(stats.elapsed() >= Duration::from_secs(2), "Should have run for at least 2 seconds");

            // TPS should be close to target (within 50% tolerance for short test)
            let target_tps = 50.0;
            let actual_tps = stats.tps();
            assert!(
                actual_tps >= target_tps * 0.5 && actual_tps <= target_tps * 1.5,
                "TPS should be close to target: expected ~{}, got {}",
                target_tps,
                actual_tps
            );

            println!("\n✓ All assertions passed!");
        }
        Err(e) => {
            println!("Workload execution failed: {}", e);
            println!("\nMake sure you have a container named 'workload-test' running:");
            println!("  dbarena create postgres --name workload-test");
            panic!("Integration test failed: {}", e);
        }
    }
}

/// Test different workload patterns
#[tokio::test]
#[ignore]
async fn test_workload_patterns() {
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => Arc::new(d),
        Err(_) => {
            println!("Docker not available - skipping test");
            return;
        }
    };

    let patterns = vec![
        WorkloadPattern::ReadHeavy,
        WorkloadPattern::WriteHeavy,
        WorkloadPattern::Oltp,
    ];

    for pattern in patterns {
        println!("\n=== Testing pattern: {:?} ===", pattern);
        println!("Description: {}", pattern.description());

        let config = WorkloadConfig {
            name: format!("{:?} Test", pattern),
            pattern: Some(pattern),
            custom_operations: None,
            custom_queries: None,
            tables: vec!["pg_database".to_string()],
            connections: 3,
            target_tps: 30,
            duration_seconds: Some(1),
            transaction_count: None,
        };

        let engine = WorkloadEngine::new(
            "workload-test".to_string(),
            DatabaseType::Postgres,
            config,
            docker.clone(),
        );

        match engine.run().await {
            Ok(stats) => {
                println!("Total: {}, Success: {}, TPS: {:.2}",
                    stats.total(),
                    stats.success_count(),
                    stats.tps()
                );

                let ops = stats.operation_counts();
                let total_ops: u64 = ops.values().sum();
                println!("Operation distribution:");
                for (op, count) in ops {
                    let percentage = (count as f64 / total_ops as f64) * 100.0;
                    println!("  {}: {} ({:.1}%)", op, count, percentage);
                }
            }
            Err(e) => {
                println!("Pattern test failed: {}", e);
            }
        }
    }
}

/// Test rate limiting accuracy
#[tokio::test]
#[ignore]
async fn test_rate_limiting_accuracy() {
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => Arc::new(d),
        Err(_) => {
            println!("Docker not available - skipping test");
            return;
        }
    };

    let target_tps = 100;
    let duration_secs = 5;

    let config = WorkloadConfig {
        name: "Rate Limiting Test".to_string(),
        pattern: Some(WorkloadPattern::Balanced),
        custom_operations: None,
        custom_queries: None,
        tables: vec!["pg_database".to_string()],
        connections: 10,
        target_tps,
        duration_seconds: Some(duration_secs),
        transaction_count: None,
    };

    let engine = WorkloadEngine::new(
        "workload-test".to_string(),
        DatabaseType::Postgres,
        config,
        docker,
    );

    println!("\n=== Rate Limiting Accuracy Test ===");
    println!("Target: {} TPS for {} seconds", target_tps, duration_secs);

    match engine.run().await {
        Ok(stats) => {
            let actual_tps = stats.tps();
            let expected_total = target_tps * duration_secs as usize;
            let actual_total = stats.total() as usize;

            println!("Expected ~{} transactions, got {}", expected_total, actual_total);
            println!("Expected {} TPS, got {:.2} TPS", target_tps, actual_tps);

            let error_percentage = ((actual_tps - target_tps as f64).abs() / target_tps as f64) * 100.0;
            println!("Error: {:.2}%", error_percentage);

            // Rate limiting should be accurate within 10%
            assert!(
                error_percentage < 10.0,
                "Rate limiting error too high: {:.2}%",
                error_percentage
            );

            println!("✓ Rate limiting accuracy within tolerance!");
        }
        Err(e) => {
            panic!("Rate limiting test failed: {}", e);
        }
    }
}

/// Test concurrent workers
#[tokio::test]
#[ignore]
async fn test_concurrent_workers() {
    let docker = match Docker::connect_with_local_defaults() {
        Ok(d) => Arc::new(d),
        Err(_) => {
            println!("Docker not available - skipping test");
            return;
        }
    };

    // Test with varying worker counts
    let worker_counts = vec![1, 5, 10, 20];

    for workers in worker_counts {
        println!("\n=== Testing with {} workers ===", workers);

        let config = WorkloadConfig {
            name: format!("{} Workers Test", workers),
            pattern: Some(WorkloadPattern::Balanced),
            custom_operations: None,
            custom_queries: None,
            tables: vec!["pg_database".to_string()],
            connections: workers,
            target_tps: 50,
            duration_seconds: Some(2),
            transaction_count: None,
        };

        let engine = WorkloadEngine::new(
            "workload-test".to_string(),
            DatabaseType::Postgres,
            config,
            docker.clone(),
        );

        match engine.run().await {
            Ok(stats) => {
                println!("Total: {}, TPS: {:.2}, Success rate: {:.2}%",
                    stats.total(),
                    stats.tps(),
                    stats.success_rate()
                );
            }
            Err(e) => {
                println!("Worker test failed: {}", e);
            }
        }
    }
}
