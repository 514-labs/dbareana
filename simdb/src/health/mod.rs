mod checker;
mod implementations;

pub use checker::HealthChecker;
pub use implementations::{MySQLHealthChecker, PostgresHealthChecker, SQLServerHealthChecker};

use crate::{Result, SimDbError};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info};

const HEALTH_CHECK_INTERVAL: Duration = Duration::from_millis(250);

pub async fn wait_for_healthy(
    container_id: &str,
    checker: &dyn HealthChecker,
    timeout: Duration,
) -> Result<()> {
    info!("Waiting for container {} to become healthy", container_id);

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Waiting for database to be ready...");

    let start = Instant::now();
    let mut attempts = 0;

    loop {
        attempts += 1;
        pb.set_message(format!(
            "Waiting for database to be ready... (attempt {})",
            attempts
        ));

        match checker.check(container_id).await {
            Ok(true) => {
                pb.finish_with_message("Database is healthy and ready!");
                info!(
                    "Container {} is healthy after {:.2}s",
                    container_id,
                    start.elapsed().as_secs_f64()
                );
                return Ok(());
            }
            Ok(false) => {
                debug!("Health check returned false, retrying...");
            }
            Err(e) => {
                debug!("Health check error (will retry): {}", e);
            }
        }

        if start.elapsed() >= timeout {
            pb.finish_with_message("Timeout waiting for database");
            return Err(SimDbError::HealthCheckTimeout(timeout.as_secs()));
        }

        sleep(HEALTH_CHECK_INTERVAL).await;
    }
}
