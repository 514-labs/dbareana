use dbarena::container::{ContainerConfig, ContainerManager, DatabaseType, DockerClient};
use dbarena::health::{wait_for_healthy, PostgresHealthChecker};
use std::time::{Duration, Instant};

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn bench_postgres_warm_start() {
    let client = DockerClient::new().expect("Failed to create Docker client");
    client
        .verify_connection()
        .await
        .expect("Docker not available");

    // Ensure image is cached
    let image = "postgres:16";
    client
        .ensure_image(image)
        .await
        .expect("Failed to ensure image");

    let manager = ContainerManager::new(client);

    // Measure creation + start + health check time
    let start = Instant::now();

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("bench-postgres-warm".to_string());

    let container = manager
        .create_container(config)
        .await
        .expect("Failed to create container");

    manager
        .start_container(&container.id)
        .await
        .expect("Failed to start container");

    let checker = PostgresHealthChecker::new(DockerClient::new().unwrap().docker().clone());
    wait_for_healthy(&container.id, &checker, Duration::from_secs(60))
        .await
        .expect("Health check failed");

    let elapsed = start.elapsed();

    // Cleanup
    manager
        .destroy_container(&container.id, false)
        .await
        .expect("Failed to destroy container");

    println!("\n🎯 PostgreSQL Warm Start: {:.2}s", elapsed.as_secs_f64());
    println!("   Target: <5s");

    // Assert performance target
    assert!(
        elapsed.as_secs() < 10,
        "Warm start took {:.2}s, which exceeds 10s threshold (target: <5s)",
        elapsed.as_secs_f64()
    );
}

#[tokio::test]
#[ignore]
async fn bench_container_destruction() {
    let client = DockerClient::new().expect("Failed to create Docker client");
    client
        .verify_connection()
        .await
        .expect("Docker not available");

    let manager = ContainerManager::new(client);

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("bench-destruction".to_string());

    let container = manager
        .create_container(config)
        .await
        .expect("Failed to create container");

    manager
        .start_container(&container.id)
        .await
        .expect("Failed to start container");

    // Measure destruction time
    let start = Instant::now();
    manager
        .destroy_container(&container.id, false)
        .await
        .expect("Failed to destroy container");
    let elapsed = start.elapsed();

    println!("\n🎯 Container Destruction: {:.2}s", elapsed.as_secs_f64());
    println!("   Target: <3s");

    assert!(
        elapsed.as_secs() < 5,
        "Destruction took {:.2}s, which exceeds 5s threshold (target: <3s)",
        elapsed.as_secs_f64()
    );
}

#[tokio::test]
#[ignore]
async fn bench_health_check_detection() {
    let client = DockerClient::new().expect("Failed to create Docker client");
    client
        .verify_connection()
        .await
        .expect("Docker not available");

    // Ensure image is cached
    client
        .ensure_image("postgres:16")
        .await
        .expect("Failed to ensure image");

    let manager = ContainerManager::new(client);

    let config = ContainerConfig::new(DatabaseType::Postgres)
        .with_name("bench-health-check".to_string());

    let container = manager
        .create_container(config)
        .await
        .expect("Failed to create container");

    manager
        .start_container(&container.id)
        .await
        .expect("Failed to start container");

    // Measure health check time
    let start = Instant::now();
    let checker = PostgresHealthChecker::new(DockerClient::new().unwrap().docker().clone());
    wait_for_healthy(&container.id, &checker, Duration::from_secs(60))
        .await
        .expect("Health check failed");
    let elapsed = start.elapsed();

    // Cleanup
    manager
        .destroy_container(&container.id, false)
        .await
        .expect("Failed to destroy container");

    println!(
        "\n🎯 Health Check Detection: {:.2}s",
        elapsed.as_secs_f64()
    );
    println!("   Target: <5s");

    assert!(
        elapsed.as_secs() < 10,
        "Health check took {:.2}s, which exceeds 10s threshold (target: <5s)",
        elapsed.as_secs_f64()
    );
}
